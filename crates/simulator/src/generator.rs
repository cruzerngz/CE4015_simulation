//! Random variable generators, their parameters and other sampling stuff are defined here.

use std::path::Iter;

use probability::{
    distribution::{self, Distribution, Inverse, Sample},
    sampler::Independent,
    source::{self, Source},
};

use crate::{
    event::{BaseStationIdx, CellEvent, CellEventType, RelativeVehiclePosition, VehicleDirection},
    FloatingPoint,
};

/// Average velocity in km/h.
pub const VEHICLE_VELOCITY_MEAN: FloatingPoint = 120.072;

/// Standard deviation of velocity in km/h.
pub const VEHICLE_VELOCITY_STDDEV: FloatingPoint = 9.0186;

/// Cell tower distributions.
pub const CELL_TOWER_DIST: (FloatingPoint, FloatingPoint) = (0.0, 20.0);

/// Location distribution inside a cell tower's coverage, in meters, from west to east.
pub const VEHICLE_LOC_DIST: (FloatingPoint, FloatingPoint) = (0.0, 2000.0);

/// Vehicle direction distribution.
pub const VEHICLE_DIR_DIST: (FloatingPoint, FloatingPoint) = (0.0, 1.0);

/// Average call duration in seconds.
pub const CALL_DURATION_LAMBDA: FloatingPoint = 99.83189;

/// Average call duration shift parameter.
pub const CALL_DURATION_LOC: FloatingPoint = 10.004;

/// Average call inter-arrival time in seconds.
pub const CALL_INTER_ARR_LAMBDA: FloatingPoint = 1.36982;

/// Exponential distribution with a location parameter.
#[derive(Clone, Debug)]
pub struct ExponentialLoc {
    inner: distribution::Exponential,
    loc: f64,
}

/// Generator iterator for call events.
#[derive(Debug)]
pub struct CallEventGenerator<S>
where
    S: Source,
{
    source: S,

    time: FloatingPoint,

    /// Event index
    count: usize,

    /// Simulation run number
    run: usize,

    // expon dist
    call_duration: SingleVariateIterator<ExponentialLoc, S>,

    // expon dist
    call_inter_arrival: SingleVariateIterator<distribution::Exponential, S>,

    // uniform dist
    cell_tower: SingleVariateIterator<distribution::Uniform, S>,
    // norm dist
    vehicle_velocity: SingleVariateIterator<distribution::Gaussian, S>,
    // uniform dist
    vehicle_position: SingleVariateIterator<distribution::Uniform, S>,
    // uniform dist
    vehicle_direction: SingleVariateIterator<distribution::Uniform, S>,
}

/// Generator iterator for call events with antithetic pair sampling.
pub struct AntitheticCallEventGenerator<S>
where
    S: Source,
{
    source: S,

    time_a: FloatingPoint,
    time_b: FloatingPoint,

    call_duration: AntitheticIterator<ExponentialLoc, S>,
    call_inter_arrival: AntitheticIterator<distribution::Exponential, S>,
    cell_tower: AntitheticIterator<distribution::Uniform, S>,
    vehicle_velocity: AntitheticIterator<distribution::Gaussian, S>,
    vehicle_position: AntitheticIterator<distribution::Uniform, S>,
    vehicle_direction: AntitheticIterator<distribution::Uniform, S>,
}

/// A generator that generates random variables from some inner distribution.
#[derive(Clone, Debug)]
pub struct SingleVariateIterator<D, S>
where
    D: Sample,
    S: Source,
{
    source: S,
    distribution: D,
}

#[derive(Clone, Debug)]
pub struct AntitheticIterator<D, S>
where
    D: Sample,
    S: Source,
{
    source: S,
    distribution: D,
}

/// An anththetic sampler that yields 2 samples from a reference sampler.
///
/// Any further samples will return the same value as the second sample.
#[derive(Debug)]
struct AntitheticSampler<'s, S>
where
    S: Source,
{
    source: &'s mut S,
    first_drawn: Option<u64>,
}

impl Distribution for ExponentialLoc {
    type Value = f64;

    fn distribution(&self, x: f64) -> f64 {
        self.inner.distribution(x - self.loc)
    }
}

impl Sample for ExponentialLoc {
    fn sample<S>(&self, source: &mut S) -> Self::Value
    where
        S: Source,
    {
        self.inner.sample(source) + self.loc
    }
}

impl ExponentialLoc {
    pub fn new(lambda: f64, loc: f64) -> Self {
        Self {
            inner: distribution::Exponential::new(lambda),
            loc,
        }
    }
}

impl<'s, S> AntitheticSampler<'s, S>
where
    S: Source,
{
    pub fn new(source: &'s mut S) -> Self {
        Self {
            source,
            first_drawn: None,
        }
    }
}

impl<'s, S> Source for AntitheticSampler<'s, S>
where
    S: Source,
{
    fn read_u64(&mut self) -> u64 {
        match self.first_drawn {
            Some(first) => u64::MAX - first,
            None => {
                let first = self.source.read_u64();
                self.first_drawn = Some(first);
                first
            }
        }
    }
}

impl<D, S> SingleVariateIterator<D, S>
where
    D: Sample + Clone,
    S: Source + Clone,
{
    pub fn new(distribution: D, mut source: S) -> Self {
        Self {
            source,
            distribution,
        }
    }

    /// Create a new iterator that generates antithetic pairs from the distribution.
    pub fn antithetic_iter(&self) -> AntitheticIterator<D, S> {
        AntitheticIterator {
            source: self.source.clone(),
            distribution: self.distribution.clone(),
        }
    }
}

/// Calculate the time to next station.
/// If the vehicle ends the call at the current station, returns None.
pub fn calculate_ttn(
    // remaining call duration
    call_dur: FloatingPoint,
    // current vehicle pos, measured from west to east
    vehicle_position: FloatingPoint,
    // vehicle velocity, km/h
    vehicle_velocity: FloatingPoint,
    // vehicle direction
    vehicle_direction: VehicleDirection,
) -> Option<FloatingPoint> {
    let dur_to_next = match vehicle_direction {
        VehicleDirection::WestToEast => {
            let remaining = VEHICLE_LOC_DIST.1 - vehicle_position;
            // convert to m/s
            remaining / (vehicle_velocity / 3.6)
        }
        VehicleDirection::EastToWest => {
            let remaining = vehicle_position;
            // convert to m/s
            remaining / (vehicle_velocity / 3.6)
        }
    };

    // validate if the call will end at the current station
    match dur_to_next <= call_dur {
        true => Some(dur_to_next as FloatingPoint),
        false => None,
    }
}

fn cell_event_from_random_variables(
    idx: usize,
    run: u32,
    call_dur: FloatingPoint,
    // need to add with previous time
    arr_time: FloatingPoint,
    cell_tower: FloatingPoint,
    vehicle_velocity: FloatingPoint,
    vehicle_position: FloatingPoint,
    vehicle_direction: FloatingPoint,
) -> CellEvent {
    let dir = match vehicle_direction > 0.5 {
        true => VehicleDirection::WestToEast,
        false => VehicleDirection::EastToWest,
    };

    let ttn = calculate_ttn(call_dur, vehicle_position, vehicle_velocity, dir);

    CellEvent {
        idx,
        run,
        time: arr_time as FloatingPoint,
        ty: CellEventType::Initiate,
        remaining_time: call_dur as FloatingPoint,
        ttn,
        velocity: vehicle_velocity as FloatingPoint,
        direction: dir,
        station: {
            let station_idx = (cell_tower % 20.0).floor() as usize;
            let station: BaseStationIdx = unsafe { std::mem::transmute(station_idx) };

            station
        },
        position: RelativeVehiclePosition::Other(vehicle_position as FloatingPoint),
    }
}

impl<S> Iterator for CallEventGenerator<S>
where
    S: Source + Clone,
{
    type Item = CellEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let call_dur = self.call_duration.clone().take(1).last()?;
        let inter_arr = self.call_inter_arrival.clone().take(1).last()?;
        let cell_tower = self.cell_tower.clone().take(1).last()?;
        let vehicle_velocity = self.vehicle_velocity.clone().take(1).last()?;
        let vehicle_position = self.vehicle_position.clone().take(1).last()?;
        let vehicle_direction = self.vehicle_direction.clone().take(1).last()?;

        self.count += 1;
        self.time += inter_arr as FloatingPoint;

        let ev = cell_event_from_random_variables(
            self.count,
            self.run as u32,
            call_dur as FloatingPoint,
            self.time,
            cell_tower as FloatingPoint,
            vehicle_velocity as FloatingPoint,
            vehicle_position as FloatingPoint,
            vehicle_direction as FloatingPoint,
        );

        Some(ev)
    }
}

impl<S> Iterator for AntitheticCallEventGenerator<S>
where
    S: Source,
{
    type Item = (CellEvent, CellEvent);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<D, S> Iterator for SingleVariateIterator<D, S>
where
    D: Sample,
    S: Source,
{
    type Item = D::Value;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.distribution.sample(&mut self.source))
    }
}

impl<D, S> Iterator for AntitheticIterator<D, S>
where
    D: Sample + Inverse,
    S: Source + Clone,
{
    type Item = (D::Value, D::Value);

    fn next(&mut self) -> Option<Self::Item> {
        let mut anti_sampler = AntitheticSampler::new(&mut self.source);

        let sample_a = self.distribution.sample(&mut anti_sampler);
        let sample_b = self.distribution.sample(&mut anti_sampler);

        Some((sample_a, sample_b))
    }
}

impl<S> CallEventGenerator<S>
where
    S: Source + Clone,
{
    /// Initialize the event generator, along with any distribution overrides.
    pub fn new(
        run: usize,
        source: S,
        call_dur: Option<ExponentialLoc>,
        inter_arrival: Option<distribution::Exponential>,
        cell_tower: Option<distribution::Uniform>,
        vehicle_velocity: Option<distribution::Gaussian>,
        vehicle_position: Option<distribution::Uniform>,
        vehicle_direction: Option<distribution::Uniform>,
    ) -> Self {
        Self {
            count: 0,
            source: source.clone(),
            time: 0.0,
            run,
            call_duration: SingleVariateIterator::new(
                call_dur.unwrap_or(ExponentialLoc::new(
                    1.0 / CALL_DURATION_LAMBDA as f64,
                    CALL_DURATION_LOC as f64,
                )),
                source.clone(),
            ),
            call_inter_arrival: SingleVariateIterator::new(
                inter_arrival.unwrap_or(distribution::Exponential::new(
                    1.0 / CALL_INTER_ARR_LAMBDA as f64,
                )),
                source.clone(),
            ),
            cell_tower: SingleVariateIterator::new(
                cell_tower.unwrap_or(distribution::Uniform::new(
                    CELL_TOWER_DIST.0 as f64,
                    CELL_TOWER_DIST.1 as f64,
                )),
                source.clone(),
            ),
            vehicle_velocity: SingleVariateIterator::new(
                vehicle_velocity.unwrap_or(distribution::Gaussian::new(
                    VEHICLE_VELOCITY_MEAN as f64,
                    VEHICLE_VELOCITY_STDDEV as f64,
                )),
                source.clone(),
            ),
            vehicle_position: SingleVariateIterator::new(
                vehicle_position.unwrap_or(distribution::Uniform::new(
                    VEHICLE_LOC_DIST.0 as f64,
                    VEHICLE_LOC_DIST.1 as f64,
                )),
                source.clone(),
            ),
            vehicle_direction: SingleVariateIterator::new(
                vehicle_direction.unwrap_or(distribution::Uniform::new(
                    VEHICLE_DIR_DIST.0 as f64,
                    VEHICLE_DIR_DIST.1 as f64,
                )),
                source.clone(),
            ),
        }
    }

    /// Create a new generator that generates antithetic pairs of samples.
    pub fn antithetic(&self) -> AntitheticCallEventGenerator<S> {
        AntitheticCallEventGenerator {
            source: self.source.clone(),
            time_a: self.time,
            time_b: self.time,
            call_duration: self.call_duration.antithetic_iter(),
            call_inter_arrival: self.call_inter_arrival.antithetic_iter(),
            cell_tower: self.cell_tower.antithetic_iter(),
            vehicle_velocity: self.vehicle_velocity.antithetic_iter(),
            vehicle_position: self.vehicle_position.antithetic_iter(),
            vehicle_direction: self.vehicle_direction.antithetic_iter(),
        }
    }
}

#[cfg(test)]
mod tests {

    use probability::distribution::Uniform;

    use super::*;

    struct MockSource(u64);

    impl Source for MockSource {
        fn read_u64(&mut self) -> u64 {
            self.0
        }
    }

    #[test]
    fn test_antithetic_iter() {
        let mut gen = SingleVariateIterator::new(Uniform::new(0.0, 10.0), source::default(0));

        for sample in gen.clone().take(10) {
            println!("{:?}", sample);
        }

        let sum = gen.clone().take(1000).sum::<f64>();
        let avg = sum / 1000.0;
        println!("average: {}", avg);

        let antithetic = gen.antithetic_iter();

        // for uniform anthithetic pairs, the average between pairs should be around 5
        let sum = antithetic
            .clone()
            .take(1000)
            .map(|(a, b)| (a + b) / 2.0)
            .sum::<f64>();

        let avg = sum / 1000.0;
        println!("average: {}", avg);

        let v = antithetic
            .clone()
            .take(10)
            //     .map(|(a, b)| (a + b) / 2.0)
            .collect::<Vec<_>>();

        println!("{:#?}", v);
    }

    #[test]
    fn test_calc_ttn() {
        let ttn = calculate_ttn(10.0, 1000.0, 100.0, VehicleDirection::EastToWest);
        assert_eq!(ttn, None);

        let ttn = calculate_ttn(100.0, 1000.0, 100.0, VehicleDirection::EastToWest);
        assert_eq!(ttn, Some(36.0));

        let ttn = calculate_ttn(
            100.0,
            RelativeVehiclePosition::WestEnd.to_float(),
            100.0,
            VehicleDirection::WestToEast,
        );
        assert_eq!(ttn, Some(72.0));

        let ttn = calculate_ttn(
            100.0,
            RelativeVehiclePosition::EastEnd.to_float(),
            100.0,
            VehicleDirection::EastToWest,
        );
        assert_eq!(ttn, Some(72.0));
    }

    #[test]
    fn test_asd() {
        let exp_dist = distribution::Exponential::new(1.0 / CALL_DURATION_LAMBDA as f64);
        let res = exp_dist.distribution(CALL_DURATION_LAMBDA as f64);

        println!("distribution: {}", res);

        let s = exp_dist.sample(&mut MockSource(u64::MAX / 2));

        println!("sample: {}", s);
    }
}
