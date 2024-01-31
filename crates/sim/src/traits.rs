//! Common traits and data structures
//!

/// Status of simulation.
#[derive(Clone, Copy, Debug)]
pub enum SimStatus {
    /// Continue processing hte simulation
    Continue,
    /// Terminate the simulation
    Stop,
}

/// Event-based simulators implement this iterator-like trait.
///
/// Each step in the simulation advances an arbitrary amount of time.
pub trait EventLike {
    /// Advance the simulation to the next enqueued event
    fn next_event(&mut self) -> SimStatus;
}

/// Process-based simulators implement this trait.
///
/// Process-based simulators run with an internal clock, in a tick-by-tick fashion.
pub trait ProcessLike {
    /// Data to be injected into the simulation, if applicable
    type Update;

    fn next_tick(&mut self, update: Self::Update) -> SimStatus;
}
