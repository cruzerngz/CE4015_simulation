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

    inner: EventType,
}

/// Inner event type
#[derive(Clone, Debug)]
enum EventType {
    /// A call is initiated by a customer
    InitiateCall,

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
///
/// Why use this instead of just a number?
/// Type safety boi
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)] // Castable to u32
pub enum BaseStation {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Eleven = 11,
    Twelve = 12,
    Thirteen = 13,
    Fourteen = 14,
    Fifteen = 15,
    Sixteen = 16,
    Seventeen = 17,
    Eighteen = 18,
    Nineteen = 19,
    Twenty = 20,
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
