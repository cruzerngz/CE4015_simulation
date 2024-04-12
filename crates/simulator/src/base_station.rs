//! Code for the base station.
//!

use serde::Serialize;

/// The base station that handles calls.
///
/// Each base station has a fixed number of available channels.
#[derive(Clone, Debug)]
pub struct BaseStation {
    /// Total number of channels available
    pub channels: usize,

    /// Available channels
    pub available_channels: usize,

    /// Channels reserved for handover requests
    pub reserved_handover_channels: Option<usize>,
    // / Channels reserved for new requestss
    // pub reserved_new_channels: Option<ChannelAllocation>,
}

// /// Channels available and used.
// ///
// /// This is used when the station enables channel reservation.
// #[derive(Clone, Debug)]
// pub struct ChannelAllocation {
//     available: usize,
//     used: usize,
// }

// /// The base station keeps track of which type of connections
// /// are being serviced.
// #[derive(Clone, Debug)]
// pub enum StationConnection {
//     New,
//     Handover,
// }

// /// The various types of termination requests a station can accept.
// #[derive(Clone, Debug)]
// pub enum StationTermination {
//     ByUser,
//     Handover,
//     ByStation,
// }

// /// Possible errors returned from making a station request.
// #[derive(Clone, Debug)]
// pub enum RequestError {
//     /// Unsuccessful new requests are blocked
//     Blocked,
//     /// Unsuccessful handover requests are terminated
//     Terminated,
// }

/// Station request
#[derive(Clone, Debug)]
pub enum StationRequest {
    Initiate,
    Terminate,
    HandoverDisconnect,
    HandoverConnect,
}

/// Channel allocation response by station
#[derive(Clone, Copy, Debug, Serialize)]
pub enum StationResponse {
    Success,
    Blocked,
    Terminated,
}

impl Default for BaseStation {
    fn default() -> Self {
        Self {
            channels: 0,
            available_channels: 0,
            reserved_handover_channels: None,
            // reserved_new_channels: None,
        }
    }
}

impl BaseStation {
    /// Create a new instance of a base station with channel reservations.
    pub fn new(channels: usize, reserved_handover: Option<usize>) -> Self {
        Self {
            channels,
            available_channels: channels,
            reserved_handover_channels: reserved_handover,
        }
    }

    /// Process an incoming request.
    pub fn process_request(&mut self, req: StationRequest) -> StationResponse {
        match req {
            StationRequest::Initiate => match self.reserved_handover_channels {
                Some(reserved) => {
                    if self.available_channels <= reserved {
                        StationResponse::Blocked
                    } else {
                        self.available_channels -= 1;
                        StationResponse::Success
                    }
                }
                None => {
                    if self.available_channels > 0 {
                        self.available_channels -= 1;
                        StationResponse::Success
                    } else {
                        StationResponse::Blocked
                    }
                }
            },

            StationRequest::Terminate | StationRequest::HandoverDisconnect => {
                self.available_channels += 1;
                assert!(self.available_channels <= self.channels);
                StationResponse::Success
            }

            StationRequest::HandoverConnect => {
                if self.available_channels > 0 {
                    self.available_channels -= 1;
                    StationResponse::Success
                } else {
                    StationResponse::Terminated
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Test base station logic
    #[test]
    fn test_base_station_processing() {
        let mut base_station = BaseStation::new(10, Some(1));

        for _ in 0..9 {
            let res = base_station.process_request(StationRequest::Initiate);
            assert!(matches!(res, StationResponse::Success));
        }

        let init_into_reserve = base_station.process_request(StationRequest::Initiate);
        println!(
            "initiate call with 1 reserved slot:   {:?}",
            init_into_reserve
        );
        assert!(matches!(init_into_reserve, StationResponse::Blocked));

        let handover_from_other_station =
            base_station.process_request(StationRequest::HandoverConnect);
        assert!(matches!(
            handover_from_other_station,
            StationResponse::Success
        ));

        let station_full = base_station.process_request(StationRequest::Initiate);
        assert!(matches!(station_full, StationResponse::Blocked));

        let station_full = base_station.process_request(StationRequest::HandoverConnect);
        assert!(matches!(station_full, StationResponse::Terminated));

        let terminate = base_station.process_request(StationRequest::Terminate);
        assert!(matches!(terminate, StationResponse::Success));

        for _ in 0..9 {
            let res = base_station.process_request(StationRequest::Terminate);
            assert!(matches!(res, StationResponse::Success));
        }
    }
}
