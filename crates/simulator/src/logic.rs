//! Event processing logic

use std::collections::VecDeque;

use simulator_core::EventLike;

use crate::{
    base_station::BaseStation,
    event::{CellEvent, CellEventResult},
};

/// Process events in the simulation
#[derive(Debug)]
pub struct EventProcessor {
    /// Future events to process
    fel: VecDeque<CellEvent>,
}

/// Shared resources in the simulation
#[derive(Clone, Debug)]
pub struct Shared {
    /// Base stations in the simulation
    base_stations: [BaseStation; 20],
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            base_stations: Default::default(),
        }
    }
}

impl EventLike for EventProcessor {
    type SharedResources = Shared;

    type EventStats = CellEventResult;

    fn step(&mut self, shared: &mut Self::SharedResources) -> Option<Vec<Self::EventStats>> {
        let next_event = self.fel.pop_front()?;

        match next_event.ty {
            crate::event::CellEventType::InitiateCall => todo!(),
            crate::event::CellEventType::TerminateCall => todo!(),
            crate::event::CellEventType::HandoverCall => todo!(),
        }

        todo!()
    }
}

impl Shared {
    pub fn new(handover_reserve: usize) -> Self {
        Self {
            base_stations: {
                core::array::from_fn(|idx| {
                    BaseStation::new(Some(handover_reserve), None)
                        .expect("base station creation must not fail")
                })
            },
        }
    }
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new(events: Vec<CellEvent>) -> Self {
        Self {
            fel: {
                let mut v = VecDeque::new();
                v.extend(events);
                v
            },
        }
    }
}
