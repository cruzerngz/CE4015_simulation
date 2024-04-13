//! Event definition for the simulator

use std::{
    iter::once_with,
    ops::{Add, Div},
};

use serde::Serialize;

use crate::{base_station::StationResponse, generator::VEHICLE_LOC_DIST, FloatingPoint};

/// A discrete event in the simulator
#[derive(Clone, Copy, Debug, Serialize)]
pub struct CellEvent {
    /// Event index
    pub idx: usize,

    pub run: u32,

    /// Time of event
    pub time: FloatingPoint,

    pub ty: CellEventType,

    /// Remaining call duration.
    ///
    /// At call initiation, this will represent the total call duration.
    pub remaining_time: FloatingPoint,

    /// Time to next base station.
    ///
    /// If this is None, the call will end at the current base station.
    pub ttn: Option<FloatingPoint>,

    /// Speed of vehicle,pub  km/h
    pub velocity: FloatingPoint,

    /// Direction of vehicle
    pub direction: VehicleDirection,

    /// Station currently in range of vehicle
    #[serde(serialize_with = "serialize_base_station")]
    pub station: BaseStationIdx,

    /// Position of vehicle relative to station
    pub position: RelativeVehiclePosition,
}

/// Result of an event
#[derive(Debug, Serialize)]
pub struct CellEventResult {
    /// Event index
    pub idx: usize,

    /// Simulation run number
    pub run: u32,

    /// Time of the event, in seconds from the start of the simulation
    pub time: FloatingPoint,

    /// Call init number, in order of initiation
    // pub call_number: u32,

    /// Event type
    pub ty: CellEventType,

    /// Indicate if the event was successful.
    ///
    /// For unsuccessful call initiations, the call is blocked.
    /// For unsuccessful call handovers, the call is dropped.
    pub outcome: StationResponse,

    pub direction: VehicleDirection,

    /// Vehicle speed
    pub speed: FloatingPoint,

    /// Base station involved in the event
    #[serde(serialize_with = "serialize_base_station")]
    pub station: BaseStationIdx,
}

/// Performance measure for sim
#[derive(Debug, Serialize)]
pub struct PerfMeasure {
    /// Percentage of blocked calls
    pub blocked_calls: FloatingPoint,

    /// Percentage of dropped calls
    pub dropped_cals: FloatingPoint,
}

/// Inner event type
#[derive(Clone, Copy, Debug, Serialize)]
pub enum CellEventType {
    /// A call is initiated by a customer
    Initiate,

    /// A call is terminated by a customer
    Terminate,

    /// A customer's call is passed from one base station to another.
    Handover,
}

/// Vehicle movement direction
#[derive(Clone, Copy, Debug, Serialize)]
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
pub enum BaseStationIdx {
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

fn serialize_base_station<S>(station: &BaseStationIdx, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let station = *station as usize;
    serializer.serialize_u32(station as u32 + 1)
}

/// Position of vehicle relative to the base station
#[derive(Clone, Copy, Debug, Serialize)]
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
        // match (self.ttn, other.ttn) {
        //     (Some(ttn1), Some(ttn2)) => ttn1 == ttn2,
        //     (None, None) => true,
        //     _ => false,
        // };

        self.time == other.time
    }
}

impl PartialOrd for CellEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.total_cmp(&other.time))
    }
}
impl Eq for CellEvent {}

impl Ord for CellEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.total_cmp(&other.time)
    }
}

impl Add<PerfMeasure> for PerfMeasure {
    type Output = PerfMeasure;

    fn add(self, rhs: PerfMeasure) -> Self::Output {
        Self {
            blocked_calls: self.blocked_calls + rhs.blocked_calls,
            dropped_cals: self.dropped_cals + rhs.dropped_cals,
        }
    }
}

impl Div<f64> for PerfMeasure {
    type Output = PerfMeasure;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            blocked_calls: self.blocked_calls / rhs as FloatingPoint,
            dropped_cals: self.dropped_cals / rhs as FloatingPoint,
        }
    }
}

impl RelativeVehiclePosition {
    /// Convert this value to the relative distance from the western end of the
    /// base station.
    pub fn to_float(&self) -> FloatingPoint {
        match self {
            RelativeVehiclePosition::WestEnd => VEHICLE_LOC_DIST.0,
            RelativeVehiclePosition::EastEnd => VEHICLE_LOC_DIST.1,
            RelativeVehiclePosition::Other(pos) => *pos,
        }
    }
}

impl CellEvent {
    pub fn to_result(&self, outcome: StationResponse) -> CellEventResult {
        CellEventResult {
            idx: self.idx,
            run: self.run,
            time: self.time,
            ty: self.ty,
            outcome,
            direction: self.direction,
            speed: self.velocity,
            station: self.station,
        }
    }
}

impl BaseStationIdx {
    pub fn next_station(&self, dir: VehicleDirection) -> Option<Self> {
        let station_idx = *self as usize;

        match dir {
            VehicleDirection::WestToEast => {
                if station_idx < 19 {
                    Some(unsafe { std::mem::transmute(station_idx + 1) })
                } else {
                    None
                }
            }
            VehicleDirection::EastToWest => {
                if station_idx > 0 {
                    Some(unsafe { std::mem::transmute(station_idx - 1) })
                } else {
                    None
                }
            }
        }
    }

    pub fn previous_station(&self, dir: VehicleDirection) -> Option<Self> {
        let station_idx = *self as usize;

        match dir {
            VehicleDirection::WestToEast => {
                if station_idx > 0 {
                    Some(unsafe { std::mem::transmute(station_idx - 1) })
                } else {
                    None
                }
            }
            VehicleDirection::EastToWest => {
                if station_idx < 19 {
                    Some(unsafe { std::mem::transmute(station_idx + 1) })
                } else {
                    None
                }
            }
        }
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

    #[test]
    fn test_base_station_next() {
        let station = BaseStationIdx::One;
        let next_station = station.next_station(VehicleDirection::WestToEast);
        assert_eq!(next_station, Some(BaseStationIdx::Two));

        let station = BaseStationIdx::Twenty;
        let next_station = station.next_station(VehicleDirection::WestToEast);
        assert_eq!(next_station, None);

        let station = BaseStationIdx::Twenty;
        let next_station = station.next_station(VehicleDirection::EastToWest);
        assert_eq!(next_station, Some(BaseStationIdx::Nineteen));

        let station = BaseStationIdx::One;
        let next_station = station.next_station(VehicleDirection::EastToWest);
        assert_eq!(next_station, None);

        let station = BaseStationIdx::Ten;
        let next_station = station.next_station(VehicleDirection::EastToWest);
        assert_eq!(next_station, Some(BaseStationIdx::Nine));
    }

    #[test]
    fn test_base_station_prev() {
        let station = BaseStationIdx::One;
        let prev_station = station.previous_station(VehicleDirection::WestToEast);
        assert_eq!(prev_station, None);

        let station = BaseStationIdx::Twenty;
        let prev_station = station.previous_station(VehicleDirection::WestToEast);
        assert_eq!(prev_station, Some(BaseStationIdx::Nineteen));

        let station = BaseStationIdx::Twenty;
        let prev_station = station.previous_station(VehicleDirection::EastToWest);
        assert_eq!(prev_station, None);

        let station = BaseStationIdx::One;
        let prev_station = station.previous_station(VehicleDirection::EastToWest);
        assert_eq!(prev_station, Some(BaseStationIdx::Two));

        let station = BaseStationIdx::Ten;
        let prev_station = station.previous_station(VehicleDirection::EastToWest);
        assert_eq!(prev_station, Some(BaseStationIdx::Eleven));

        // generate tests to check the prevoius station for stations 2-20
        for station in BaseStationIdx::Two as usize..=BaseStationIdx::Twenty as usize {
            let station: BaseStationIdx = unsafe { std::mem::transmute(station) };
            let prev_station = station.previous_station(VehicleDirection::EastToWest);

            println!("curr: {:?}, prev: {:?}", station, prev_station);
            assert_eq!(
                prev_station,
                Some(unsafe { std::mem::transmute(station as usize + 1) })
            );
        }
    }
}
