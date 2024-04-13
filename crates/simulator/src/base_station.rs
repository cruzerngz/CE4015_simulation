//! Code for the base station.
//!

use serde::Serialize;

use crate::debug_println;

/// The base station that handles calls.
///
/// Each base station has a fixed number of available channels.
#[derive(Clone, Debug, Default)]
pub struct BaseStation {
    /// Total number of channels available
    pub channels: usize,

    /// Available channels
    pub available_channels: usize,

    /// Channels reserved for handover requests
    pub reserved_handover_channels: Option<usize>,
    // / Channels reserved for new requestss
    // pub reserved_new_channels: Option<ChannelAllocation>,
    /// For validation purposes
    pub active_users: Vec<usize>,
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

impl BaseStation {
    /// Create a new instance of a base station with channel reservations.
    pub fn new(channels: usize, reserved_handover: Option<usize>) -> Self {
        Self {
            channels,
            available_channels: channels,
            reserved_handover_channels: reserved_handover,
            active_users: Vec::new(),
        }
    }

    /// Process an incoming request.
    ///
    /// Idx is used for debugging.
    pub fn process_request(&mut self, req: StationRequest, idx: usize) -> StationResponse {
        debug_println!("{:?} request from event {}", req, idx);

        let resp = match req {
            StationRequest::Initiate => match self.reserved_handover_channels {
                Some(reserved) => {
                    // debug_println!(
                    //     "reserved handover: {}, available channels: {}",
                    //     reserved, self.available_channels
                    // );
                    if self.available_channels <= reserved {
                        StationResponse::Blocked
                    } else {
                        self.available_channels -= 1;
                        self.active_users.push(idx);
                        StationResponse::Success
                    }
                }
                None => {
                    // debug_println!("available channels: {}", self.available_channels);
                    if self.available_channels > 0 {
                        self.available_channels -= 1;
                        self.active_users.push(idx);
                        StationResponse::Success
                    } else {
                        StationResponse::Blocked
                    }
                }
            },

            StationRequest::Terminate | StationRequest::HandoverDisconnect => {
                self.active_users.sort();
                match self.active_users.binary_search(&idx) {
                    Ok(found) => {
                        self.active_users.remove(found);
                    }
                    Err(_) => panic!("unknown event idx attempting termination: {}", idx),
                }

                self.available_channels += 1;
                // debug_println!(
                //     "terminate/handover disconnect. Channels available: {}",
                //     self.available_channels
                // );
                assert!(
                    self.available_channels <= self.channels,
                    "available channels ({}) exceeded limit: {}",
                    self.available_channels,
                    self.channels
                );
                StationResponse::Success
            }

            StationRequest::HandoverConnect => {
                if self.available_channels > 0 {
                    self.available_channels -= 1;
                    self.active_users.push(idx);
                    StationResponse::Success
                } else {
                    StationResponse::Terminated
                }
            }
        };

        assert_eq!(
            self.active_users.len(),
            self.channels - self.available_channels,
            "number of active users must match occupied channels"
        );

        self.active_users.sort();
        assert!(
            self.active_users.windows(2).all(|w| w[0] != w[1]),
            "no duplicate users"
        );

        debug_println!(
            "station resp: {:?}, remaining channels: {}",
            resp, self.available_channels
        );

        resp
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Test base station logic
    #[test]
    fn test_base_station_processing() {
        let mut base_station = BaseStation::new(10, Some(1));

        for idx in 0..9 {
            let res = base_station.process_request(StationRequest::Initiate, idx);
            assert!(matches!(res, StationResponse::Success));
        }

        let init_into_reserve = base_station.process_request(StationRequest::Initiate, 10);
        debug_println!(
            "initiate call with 1 reserved slot:   {:?}",
            init_into_reserve
        );
        assert!(matches!(init_into_reserve, StationResponse::Blocked));

        let handover_from_other_station =
            base_station.process_request(StationRequest::HandoverConnect, 10);
        assert!(matches!(
            handover_from_other_station,
            StationResponse::Success
        ));

        let station_full = base_station.process_request(StationRequest::Initiate, 11);
        assert!(matches!(station_full, StationResponse::Blocked));

        let station_full = base_station.process_request(StationRequest::HandoverConnect, 11);
        assert!(matches!(station_full, StationResponse::Terminated));

        let terminate = base_station.process_request(StationRequest::Terminate, 10);
        assert!(matches!(terminate, StationResponse::Success));

        for idx in 0..9 {
            let res = base_station.process_request(StationRequest::Terminate, idx);
            assert!(matches!(res, StationResponse::Success));
        }
    }
}
