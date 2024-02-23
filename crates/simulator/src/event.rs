//! Event definition for the simulator

/// A discrete event in the simulator
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// A call is initiated by a customer
    InitiateCall,

    /// A call is terminated by a customer
    TerminateCall,

    /// A customer's call is passed from one base station to another.
    HandoverCall,
}
