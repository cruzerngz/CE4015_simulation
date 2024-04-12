//! Random variable generators for various parts of the sim

use std::path::Iter;

use probability::{
    distribution::{self, Inverse, Sample},
    sampler::Independent,
    source::{self, Source},
};

use crate::event::{
    BaseStation, CellEvent, CellEventType, RelativeVehiclePosition, VehicleDirection,
};

/// Average velocity in km/h.
const VEHICLE_VELOCITY_MEAN: f64 = 120.072;

/// Standard deviation of velocity in km/h.
const VEHICLE_VELOCITY_STDDEV: f64 = 9.0186;

/// Cell tower distributions.
const CELL_TOWER_DIST: (f64, f64) = (0.0, 20.0);

/// Location distribution inside a cell tower's coverage, in meters, from west to east.
const VEHICLE_LOC_DIST: (f64, f64) = (0.0, 2000.0);

/// Vehicle direction distribution.
const VEHICLE_DIR_DIST: (f64, f64) = (0.0, 1.0);

/// Average call duration in seconds.
const CALL_DURATION_LAMBDA: f64 = 99.83189;

/// Average call inter-arrival time in seconds.
const CALL_INTER_ARR_LAMBDA: f64 = 1.36982;

/// Generator iterator for call events.
#[derive(Debug)]
pub struct CallEventGenerator<S>
where
    S: Source,
{
    source: S,

    time: f64,

    // expon dist
    call_duration: SingleVariateIterator<distribution::Exponential, S>,

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

    time_a: f64,
    time_b: f64,

    call_duration: AntitheticIterator<distribution::Exponential, S>,
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

impl<'s, S> AntitheticSampler<'s, S>
where
    // D: Sample + Inverse,
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

        self.time += inter_arr;
        let dir = match vehicle_direction > 0.5 {
            true => VehicleDirection::WestToEast,
            false => VehicleDirection::EastToWest,
        };

        // time to next station calculation
        let dur_to_next = match dir {
            VehicleDirection::WestToEast => {
                let remaining = VEHICLE_LOC_DIST.1 - vehicle_position;
                let time = remaining / (vehicle_velocity / 3.6); // convert to m/s
                time
            }
            VehicleDirection::EastToWest => {
                let remaining = vehicle_position;
                let time = remaining / (vehicle_velocity / 3.6); // convert to m/s
                time
            }
        };

        // validate if the call will end at the current station
        let ttn = match dur_to_next < call_dur {
            true => Some(dur_to_next as f32),
            false => None,
        };

        Some(CellEvent {
            time: self.time as f32,
            ty: CellEventType::InitiateCall,
            remaining_time: call_dur as f32,
            ttn,
            velocity: vehicle_velocity as f32,
            direction: dir,
            station: {
                let station_idx = (cell_tower % 20.0).floor() as usize;
                let station: BaseStation = unsafe { std::mem::transmute(station_idx) };

                station
            },
            position: RelativeVehiclePosition::Other(vehicle_position as f32),
        })
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
        source: S,
        inter_arrival: Option<distribution::Exponential>,
        cell_tower: Option<distribution::Uniform>,
        vehicle_velocity: Option<distribution::Gaussian>,
        vehicle_position: Option<distribution::Uniform>,
        vehicle_direction: Option<distribution::Uniform>,
    ) -> Self {
        Self {
            source: source.clone(),
            time: 0.0,
            call_duration: SingleVariateIterator::new(
                inter_arrival.unwrap_or(distribution::Exponential::new(1.0 / CALL_DURATION_LAMBDA)),
                source.clone(),
            ),
            call_inter_arrival: SingleVariateIterator::new(
                inter_arrival
                    .unwrap_or(distribution::Exponential::new(1.0 / CALL_INTER_ARR_LAMBDA)),
                source.clone(),
            ),
            cell_tower: SingleVariateIterator::new(
                cell_tower.unwrap_or(distribution::Uniform::new(
                    CELL_TOWER_DIST.0,
                    CELL_TOWER_DIST.1,
                )),
                source.clone(),
            ),
            vehicle_velocity: SingleVariateIterator::new(
                vehicle_velocity.unwrap_or(distribution::Gaussian::new(
                    VEHICLE_VELOCITY_MEAN,
                    VEHICLE_VELOCITY_STDDEV,
                )),
                source.clone(),
            ),
            vehicle_position: SingleVariateIterator::new(
                vehicle_position.unwrap_or(distribution::Uniform::new(
                    VEHICLE_LOC_DIST.0,
                    VEHICLE_LOC_DIST.1,
                )),
                source.clone(),
            ),
            vehicle_direction: SingleVariateIterator::new(
                vehicle_direction.unwrap_or(distribution::Uniform::new(
                    VEHICLE_DIR_DIST.0,
                    VEHICLE_DIR_DIST.1,
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
            call_inter_arrival: self.call_duration.antithetic_iter(),
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
}
