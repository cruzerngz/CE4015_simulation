//! Code for the base station.
//!

/// The base station that handles calls.
///
/// Each base station has a fixed number of available channels.
#[derive(Clone, Debug)]
pub struct BaseStation<const CHANNELS: usize> {
    /// Available channels to be used
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

/// Possible errors returned
#[derive(Clone, Debug)]
pub enum RequestError {
    /// Unsuccessful new requests are blocked
    Blocked,
    /// Unsuccessful handover requests are terminated
    Terminated,
}

impl<const CHANNELS: usize> Default for BaseStation<CHANNELS> {
    fn default() -> Self {
        Self {
            available_channels: CHANNELS,
            reserved_handover_channels: todo!(),
            reserved_new_channels: todo!(),
        }
    }
}

impl<const CHANNELS: usize> BaseStation<CHANNELS> {
    /// Process a request with no channel reservation
    pub fn process_request(&mut self, req: StationRequest) -> Result<(), RequestError> {
        match (self.available_channels, req) {
            (0, StationRequest::Connect(StationConnection::New)) => Err(RequestError::Blocked),
            (0, StationRequest::Connect(StationConnection::Handover)) => {
                Err(RequestError::Terminated)
            }

            (num, StationRequest::Connect(ty)) => Ok(()),

            _ => todo!(), // (0, StationRequest::Connect) => Err(RequestError::Blocked),
                          // (0, StationRequest::Handover) => Err(RequestError::Terminated),

                          // (_, StationRequest::Connect | StationRequest::Handover) => {
                          //     self.available_channels -= 1;
                          //     Ok(())
                          // }

                          // (_, StationRequest::Terminate) => {
                          //     self.available_channels += 1;
                          //     Ok(())
                          // }
        }
    }
}
