//! Random variable generators, their parameters and other sampling stuff are defined here.

use probability::{distribution, source::Source};
use simulator_core::{AntitheticIterator, ExponentialLoc, SingleVariateIterator};

use crate::{
    event::{BaseStationIdx, CellEvent, CellEventType, RelativeVehiclePosition, VehicleDirection},
    FloatingPoint,
};

/// Number of samples to cache for antithetic sampling.
/// Required as gaussian distributions require more than one sample call.
pub const ANTITHETIC_PREPARE: usize = 10;

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
    #[allow(dead_code)]
    source: S,

    time_a: FloatingPoint,
    time_b: FloatingPoint,

    count: usize,

    run: usize,

    call_duration: AntitheticIterator<ExponentialLoc, S>,
    call_inter_arrival: AntitheticIterator<distribution::Exponential, S>,
    cell_tower: AntitheticIterator<distribution::Uniform, S>,
    vehicle_velocity: AntitheticIterator<distribution::Gaussian, S>,
    vehicle_position: AntitheticIterator<distribution::Uniform, S>,
    vehicle_direction: AntitheticIterator<distribution::Uniform, S>,
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
    S: Source + Clone,
{
    type Item = (CellEvent, CellEvent);

    fn next(&mut self) -> Option<Self::Item> {
        // debug_println!("generating call duration");
        let (call_dur_a, call_dur_b) = self.call_duration.clone().take(1).last()?;
        // debug_println!("generating inter arrival");
        let (inter_arr_a, inter_arr_b) = self.call_inter_arrival.clone().take(1).last()?;
        // debug_println!("generating cell tower");
        let (cell_tower_a, cell_tower_b) = self.cell_tower.clone().take(1).last()?;
        // debug_println!("generating vehicle velocity");
        let (vehicle_velocity_a, vehicle_velocity_b) =
            self.vehicle_velocity.clone().take(1).last()?;
        // debug_println!("generating vehicle position");
        let (vehicle_position_a, vehicle_position_b) =
            self.vehicle_position.clone().take(1).last()?;
        // debug_println!("generating vehicle direction");
        let (vehicle_direction_a, vehicle_direction_b) =
            self.vehicle_direction.clone().take(1).last()?;

        self.count += 1;
        self.time_a += inter_arr_a as FloatingPoint;
        self.time_b += inter_arr_b as FloatingPoint;

        let ev_a = cell_event_from_random_variables(
            self.count,
            self.run as u32,
            call_dur_a as FloatingPoint,
            self.time_a,
            cell_tower_a as FloatingPoint,
            vehicle_velocity_a as FloatingPoint,
            vehicle_position_a as FloatingPoint,
            vehicle_direction_a as FloatingPoint,
        );

        let ev_b = cell_event_from_random_variables(
            self.count,
            self.run as u32,
            call_dur_b as FloatingPoint,
            self.time_b,
            cell_tower_b as FloatingPoint,
            vehicle_velocity_b as FloatingPoint,
            vehicle_position_b as FloatingPoint,
            vehicle_direction_b as FloatingPoint,
        );

        Some((ev_a, ev_b))
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
            count: self.count,
            run: self.run,
            call_duration: self.call_duration.antithetic_iter(ANTITHETIC_PREPARE),
            call_inter_arrival: self.call_inter_arrival.antithetic_iter(ANTITHETIC_PREPARE),
            cell_tower: self.cell_tower.antithetic_iter(ANTITHETIC_PREPARE),
            vehicle_velocity: self.vehicle_velocity.antithetic_iter(ANTITHETIC_PREPARE),
            vehicle_position: self.vehicle_position.antithetic_iter(ANTITHETIC_PREPARE),
            vehicle_direction: self.vehicle_direction.antithetic_iter(ANTITHETIC_PREPARE),
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {

    use probability::{
        distribution::{Distribution, Sample, Uniform},
        source,
    };

    use crate::{debug_println, RngSource};

    use super::*;

    struct MockSource(u64);

    impl Source for MockSource {
        fn read_u64(&mut self) -> u64 {
            self.0
        }
    }

    #[test]
    fn test_antithetic_iter() {
        let gen = SingleVariateIterator::new(Uniform::new(0.0, 10.0), source::default(0));

        for sample in gen.clone().take(10) {
            debug_println!("{:?}", sample);
        }

        let sum = gen.clone().take(1000).sum::<f64>();
        let avg = sum / 1000.0;
        debug_println!("average: {}", avg);

        let antithetic = gen.antithetic_iter(ANTITHETIC_PREPARE);

        // for uniform anthithetic pairs, the average between pairs should be around 5
        let sum = antithetic
            .clone()
            .take(1000)
            .map(|(a, b)| (a + b) / 2.0)
            .sum::<f64>();

        let avg = sum / 1000.0;
        debug_println!("average: {}", avg);

        let v = antithetic
            .clone()
            .take(10)
            //     .map(|(a, b)| (a + b) / 2.0)
            .collect::<Vec<_>>();

        debug_println!("{:#?}", v);
    }

    /// Gaussian generation is giving problems when generating antithetic pairs
    #[test]
    fn test_antihetic_gaussian_iter() {
        let gen = SingleVariateIterator::new(
            distribution::Gaussian::new(
                VEHICLE_VELOCITY_MEAN as f64,
                VEHICLE_VELOCITY_STDDEV as f64,
            ),
            source::default(10),
        );

        println!("creating generator iterator");
        let mut antithetic = gen.antithetic_iter(ANTITHETIC_PREPARE);

        println!("iterating");

        for _ in 0..100 {
            let v = antithetic.next().unwrap();
            println!("next pair: {:?}", v);
        }

        // let v = antithetic
        //     // .clone()
        //     .take(10)
        //     .enumerate()
        //     .map(|(idx, (a, b))| {
        //         println!("iter {}", idx);
        //         (a + b) / 2.0
        //     })
        //     .collect::<Vec<_>>();

        // debug_println!("{:#?}", v);
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
    fn test_call_event_gen() {
        let generator = CallEventGenerator::new(
            1,
            RngSource(rand::rngs::OsRng),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let num_events = 100000;
        println!("generating {} events", num_events);
        let call_events = generator.antithetic().take(num_events).collect::<Vec<_>>();
        println!("{} events generated", num_events);
    }

    #[test]
    fn test_asd() {
        let exp_dist = distribution::Exponential::new(1.0 / CALL_DURATION_LAMBDA as f64);
        let res = exp_dist.distribution(CALL_DURATION_LAMBDA as f64);

        debug_println!("distribution: {}", res);

        let s = exp_dist.sample(&mut MockSource(u64::MAX / 2));

        debug_println!("sample: {}", s);
    }
}
