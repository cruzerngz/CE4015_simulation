//! Common traits and data structures
//!

use std::path::Iter;

/// Status of simulation.
#[derive(Clone, Copy, Debug)]
pub enum SimStatus {
    /// Continue processing hte simulation
    Continue,
    /// Terminate the simulation
    Stop,
}

#[derive(Clone, Copy, Debug)]
struct Event;

#[derive(Clone, Copy, Debug)]
struct Process;

/// Event-based simulators implement this iterator-like trait.
///
/// Each step in the simulation advances an arbitrary amount of time.
pub trait EventLike: Sized {
    /// Advance the simulation to the next enqueued event
    fn next_event(&mut self) -> SimStatus;
}

/// Iterator wrapper
pub struct EventIterator<T: EventLike> {
    inner: T,
}

// Blanket trait that includes Iterator functionality
pub trait ToEventIterator: EventLike {
    fn to_event_iter(self) -> EventIterator<Self>
    where
        Self: Sized,
    {
        EventIterator { inner: self }
    }
}

// blanket impl
impl<T: EventLike> ToEventIterator for T {}

impl<T: EventLike> Iterator for EventIterator<T> {
    type Item = SimStatus;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next_event() {
            SimStatus::Continue => Some(SimStatus::Continue),
            SimStatus::Stop => None,
        }
    }
}

fn asd<T: Iterator<Item = SimStatus>>(i: T) {
    for status in i.into_iter() {
        ()
    }
}

struct MySim;
impl EventLike for MySim {
    fn next_event(&mut self) -> SimStatus {
        SimStatus::Continue
    }
}

fn def() {
    let x = MySim;

    let x_i = x.to_event_iter();

    // asd(x_i);

    let executor: SimExecutor<Event, EventIterator<MySim>> = SimExecutor {
        iter: x_i,
        _marker: std::marker::PhantomData,
    };
}

/// Process-based simulators implement this trait.
///
/// Process-based simulators run with an internal clock, in a tick-by-tick fashion.
pub trait ProcessLike: Sized {
    /// Data to be injected into the simulation, if applicable
    type Update;

    fn next_tick(&mut self, update: Self::Update) -> SimStatus;
}

pub struct SimExecutor<T, P: Iterator<Item = SimStatus>> {
    iter: P,

    _marker: core::marker::PhantomData<T>,
}

impl<Sim> SimExecutor<Event, Sim>
where
    Sim: Iterator<Item = SimStatus>,
{
    pub fn from_event(event: Sim) -> Self {
        Self {
            iter: event.into_iter(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Sim: Iterator<Item = SimStatus>> SimExecutor<Process, Sim> {
    pub fn from_process(proc: Sim) -> Self {
        Self {
            iter: proc.into_iter(),
            _marker: std::marker::PhantomData,
        }
    }
}
