//! Event definition for the simulator

use std::clone;

/// A discrete event in the simulator
#[derive(Clone, Debug)]
pub struct Event {
    /// Time when vehicle first enters either end of the highway
    time: f64,

    /// Time to next base station.
    ttn: f64,

    /// Speed of vehicle, km/h
    velocity: f64,

    /// Direction of vehicle
    direction: VehicleDirection,

    /// Station currently in range of vehicle
    station: BaseStation,

    /// Position of vehicle relative to station
    position: StationPosition,

    inner: EventType,
}

/// Inner event type
#[derive(Clone, Debug)]
enum EventType {
    /// A call is initiated by a customer
    InitiateCall {},

    /// A call is terminated by a customer
    TerminateCall,

    /// A customer's call is passed from one base station to another.
    HandoverCall,
}

/// Vehicle movement direction
#[derive(Clone, Debug)]
pub enum VehicleDirection {
    /// The vehicle is moving from left to right, and encounters base stations in
    /// ascending order.
    WestToEast,

    /// The vehicle is moving right to left, and encounters base stations in
    /// descending order.
    EastToWest,
}

/// The base stations servicing a vehicle.
/// This enum will be used to index into the base station vector.
///
/// Why use this instead of just a number?
/// Type safety boi
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BaseStation {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
    Eight = 7,
    Nine = 8,
    Ten = 9,
    Eleven = 10,
    Twelve = 11,
    Thirteen = 12,
    Fourteen = 13,
    Fifteen = 14,
    Sixteen = 15,
    Seventeen = 16,
    Eighteen = 17,
    Nineteen = 18,
    Twenty = 19,
}

/// Position of vehicle relative to the base station
#[derive(Clone, Debug)]
pub enum StationPosition {
    /// Vehicle is at the western end of the station's coverage
    /// area.
    WestEnd,
    /// Vehicle is at the eastern end of the station's coverage
    /// area.
    EastEnd,

    /// Vehicle is somewhere along the station's coverage area, measured from
    /// the west.
    Other(f64),
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
impl Eq for Event {}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.total_cmp(&other.time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_floats() {
        let mut x: Vec<Event> = Default::default();
        x.sort();
    }
}
