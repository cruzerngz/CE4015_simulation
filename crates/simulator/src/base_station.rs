//! Code for the base station.
//!

/// The base station that handles calls.
///
/// Each base station has a fixed number of available channels.
#[derive(Clone, Debug)]
pub struct BaseStation {
    /// Total number of channels available
    channels: usize,

    /// Available channels
    available_channels: usize,

    /// Channels reserved for handover requests
    reserved_handover_channels: Option<ChannelAllocation>,

    /// Channels reserved for new requestss
    reserved_new_channels: Option<ChannelAllocation>,
}

/// A request made to a base station.
///
/// A handover request is pretty much the same as a connect request.
///
/// It is a termination request for one station immediately followed by a
/// connection request at the next station.
#[derive(Clone, Debug)]
pub enum StationRequest {
    Connect(StationConnection),
    Terminate(StationConnection),
}

/// Channels available and used.
///
/// This is used when the station enables channel reservation.
#[derive(Clone, Debug)]
pub struct ChannelAllocation {
    available: usize,
    used: usize,
}

/// The base station keeps track of which type of connections
/// are being serviced.
#[derive(Clone, Debug)]
pub enum StationConnection {
    New,
    Handover,
}

/// The various types of termination requests a station can accept.
#[derive(Clone, Debug)]
pub enum StationTermination {
    ByUser,
    Handover,
    ByStation,
}

/// Possible errors returned from making a station request.
#[derive(Clone, Debug)]
pub enum RequestError {
    /// Unsuccessful new requests are blocked
    Blocked,
    /// Unsuccessful handover requests are terminated
    Terminated,
}

impl Default for BaseStation {
    fn default() -> Self {
        Self {
            channels: 0,
            available_channels: 0,
            reserved_handover_channels: None,
            reserved_new_channels: None,
        }
    }
}

impl BaseStation {
    /// Create a new instance of a base station with channel reservations.
    pub fn new(reserved_handover: Option<usize>, reserved_new: Option<usize>) -> Result<Self, ()> {
        let reserved_sum = reserved_handover.unwrap_or(0) + reserved_new.unwrap_or(0);

        if reserved_sum > 0 {
            return Err(());
        }

        Ok(Self {
            channels: 0,
            available_channels: 0,
            reserved_handover_channels: match reserved_handover {
                Some(num_reserved) => Some(ChannelAllocation {
                    available: num_reserved,
                    used: 0,
                }),
                None => None,
            },
            reserved_new_channels: match reserved_new {
                Some(num_reserved) => Some(ChannelAllocation {
                    available: num_reserved,
                    used: 0,
                }),
                None => None,
            },
        })
    }

    /// Process an incoming request.
    pub fn process_request(&mut self, req: StationRequest) -> Result<(), RequestError> {
        match (self.available_channels, req) {
            (0, StationRequest::Connect(StationConnection::New)) => Err(RequestError::Blocked),
            (0, StationRequest::Connect(StationConnection::Handover)) => {
                Err(RequestError::Terminated)
            }

            (num, StationRequest::Connect(ty)) => Ok(()),

            (num, StationRequest::Terminate(ty)) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_base_station_processing() {
        panic!("NOT IMPLEEEMENTUD")
    }
}
