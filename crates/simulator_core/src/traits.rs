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
    /// The result of stepping through one event.
    ///
    /// This type should contain details about the event that was just processed,
    /// like the values of any random variables, etc.s
    type EventResult;

    type Event;

    /// Advance the simulation to the next enqueued event
    fn next_event(&mut self) -> Option<Self::EventResult>;

    /// Initialize the discrete event simulator with a set of initial events
    fn initialize_initial_events<E: AsRef<[Self::Event]>>(&mut self, initial_ev: E);
}

/// Iterator wrapper
#[derive(Clone)]
pub struct EventIterator<T>
where
    T: EventLike,
{
    inner: T,
}

impl<T> From<T> for EventIterator<T>
where
    T: EventLike,
{
    fn from(value: T) -> Self {
        value.to_event_iter()
    }
}

impl<T> EventIterator<T>
where
    T: EventLike,
{
    /// Initialize the simulator with a sequence of initial events
    pub fn with_initial_events<E>(mut self, ev: E) -> Self
    where
        E: AsRef<[<T as EventLike>::Event]>,
    {
        self.inner.initialize_initial_events(ev);

        self
    }
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
impl<T> ToEventIterator for T where T: EventLike {}

impl<T: EventLike> Iterator for EventIterator<T> {
    type Item = <T as EventLike>::EventResult;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_event()
    }
}

/// The runner for the sim
#[derive(Clone)]
pub struct SimExecutor<T, P: Iterator<Item = SimStatus> + Clone> {
    iter: P,

    _marker: core::marker::PhantomData<T>,
}

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
