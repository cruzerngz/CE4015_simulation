
/// Status of simulation.
#[derive(Clone, Copy, Debug)]
pub enum SimStatus {
    /// Continue processing hte simulation
    Continue,
    /// Terminate the simulation
    Stop,
}

#[derive(Clone, Copy, Debug)]
pub struct Event;

#[derive(Clone, Copy, Debug)]
pub struct Process;

/// Event-based simulators implement this iterator-like trait.
///
/// Each step in the simulation advances an arbitrary amount of time.
pub trait EventLike: Sized {
    /// Advance the simulation to the next enqueued event
    fn next_event(&mut self) -> SimStatus;
}

/// Iterator wrapper
#[derive(Clone)]
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

/// Process-based simulators implement this trait.
///
/// Process-based simulators run with an internal clock, in a tick-by-tick fashion.
pub trait ProcessLike: Sized {
    /// Data to be injected into the simulation, if applicable
    type Update;

    fn next_tick(&mut self, update: Self::Update) -> SimStatus;
}

/// The runner for the sim
#[derive(Clone)]
pub struct SimExecutor<T, P: Iterator<Item = SimStatus> + Clone> {
    iter: P,

    _marker: core::marker::PhantomData<T>,
}

// impl<Sim, I> SimExecutor<Event, I>
// where
//     Sim: ToEventIterator,
//     I: Iterator<Item = SimStatus>
// {

// }

// impl<Sim> SimExecutor<Event, Sim>
// where
//     Sim: Iterator<Item = SimStatus> + Clone,
// {
//     pub fn from_event(event: Sim) -> Self {
//         Self {
//             iter: event,
//             _marker: std::marker::PhantomData,
//         }
//     }
// }

// impl<Sim: Iterator<Item = SimStatus> + Clone> SimExecutor<Process, Sim> {
//     pub fn from_process(proc: Sim) -> Self {
//         Self {
//             iter: proc,
//             _marker: std::marker::PhantomData,
//         }
//     }
// }

impl<T, I> SimExecutor<T, I>
where
    I: Iterator<Item = SimStatus> + Clone,
{
    /// Construct an executor from an iterator that yields [SimStatus]
    pub fn from_iterator(iter: I) -> Self {
        Self {
            iter,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, Sim> SimExecutor<T, Sim>
where
    Sim: Iterator<Item = SimStatus> + Clone,
{
    /// Runs the simulation to completion
    pub fn execute(&mut self) -> SimStatus {
        let mut final_step = SimStatus::Continue;

        for step in self.iter.clone() {
            final_step = step;
        }

        final_step
    }
}
