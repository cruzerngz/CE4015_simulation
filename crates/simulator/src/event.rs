//! Event definition for the simulator

use serde::Serialize;

/// Common float type for the simulator
type FloatingPoint = f32;

/// A discrete event in the simulator
#[derive(Clone, Debug)]
pub struct CellEvent {
    /// Time of event
    pub time: FloatingPoint,
    pub ty: CellEventType,

    /// Time to next base station.
    pub ttn: FloatingPoint,

    /// Speed of vehicle,pub  km/h
    pub velocity: FloatingPoint,

    /// Direction of vehicle
    pub direction: VehicleDirection,

    /// Station currently in range of vehicle
    pub station: BaseStation,

    /// Position of vehicle relative to station
    pub position: RelativeVehiclePosition,
}

/// Result of an event
#[derive(Debug, Serialize)]
pub struct CellEventResult {
    /// Simulation run number
    run: u32,

    /// Time of the event, in seconds from the start of the simulation
    time: FloatingPoint,

    /// Call init number, in order of initiation
    call_number: u32,

    /// Event type
    ty: CellEventType,

    direction: VehicleDirection,

    /// Vehicle speed
    speed: FloatingPoint,

    /// Base station involved in the event
    #[serde(serialize_with = "serialize_base_station")]
    station: BaseStation,
}

/// Inner event type
#[derive(Clone, Debug, Serialize)]
pub enum CellEventType {
    /// A call is initiated by a customer
    InitiateCall,

    /// A call is terminated by a customer
    TerminateCall,

    /// A customer's call is passed from one base station to another.
    HandoverCall,
}

/// Vehicle movement direction
#[derive(Clone, Debug, Serialize)]
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
#[repr(usize)]
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

fn serialize_base_station<S>(station: &BaseStation, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let station = *station as usize;
    serializer.serialize_u32(station as u32 + 1)
}

/// Position of vehicle relative to the base station
#[derive(Clone, Debug)]
pub enum RelativeVehiclePosition {
    /// Vehicle is at the western end of the station's coverage
    /// area.
    WestEnd,
    /// Vehicle is at the eastern end of the station's coverage
    /// area.
    EastEnd,

    /// Vehicle is somewhere along the station's coverage area, measured from
    /// west to east, in meters.
    Other(FloatingPoint),
}

impl PartialEq for CellEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for CellEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
impl Eq for CellEvent {}

impl Ord for CellEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.total_cmp(&other.time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_floats() {
        let mut x: Vec<CellEvent> = Default::default();
        x.sort();
    }
}
