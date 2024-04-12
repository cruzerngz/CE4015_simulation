//! Event processing logic

use std::collections::VecDeque;

use simulator_core::EventLike;

use crate::{
    base_station::{BaseStation, StationRequest, StationResponse},
    event::{CellEvent, CellEventResult, CellEventType, PerfMeasure},
};

/// Process events in the simulation
#[derive(Debug)]
pub struct EventProcessor {
    run_num: usize,
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

    type PerformanceMeasure = PerfMeasure;

    fn step(&mut self, shared: &mut Self::SharedResources) -> Option<Vec<Self::EventStats>> {
        let next_event = self.fel.pop_front()?;

        let results = match next_event.ty {
            crate::event::CellEventType::InitiateCall => {
                self.process_call_initiation(next_event, shared)
            }
            crate::event::CellEventType::TerminateCall => {
                self.process_call_terminate(next_event, shared)
            }
            crate::event::CellEventType::HandoverCall => {
                self.process_call_handover(next_event, shared)
            }
        };

        Some(results)
    }

    fn calculate_performance_measure(results: &[Self::EventStats]) -> Self::PerformanceMeasure {
        todo!()
    }
}

impl Shared {
    pub fn new(handover_reserve: usize) -> Self {
        Self {
            base_stations: {
                core::array::from_fn(|idx| BaseStation::new(10, Some(handover_reserve)))
            },
        }
    }
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new(run_num: usize, events: Vec<CellEvent>) -> Self {
        Self {
            run_num,
            fel: {
                let mut v = VecDeque::new();
                v.extend(events);
                v
            },
        }
    }

    /// Should not be needed
    #[cfg(notset)]
    fn sort_inner(&mut self) {
        self.fel.rotate_right(self.fel.as_slices().1.len());
        assert!(self.fel.as_slices().1.is_empty());
        self.fel.as_mut_slices().0.sort();
    }

    /// Insert an event in sorted order into the queue
    pub fn insert_event(&mut self, event: CellEvent) {
        match self.fel.binary_search(&event) {
            Ok(pos) | Err(pos) => {
                self.fel.insert(pos, event);
            }
        }
    }

    // event parameter must be a call initiation event
    fn process_call_initiation(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::InitiateCall));

        // check shared resource

        let station = shared
            .base_stations
            .get_mut(event.station as usize)
            .expect("station must exist");

        let response = station.process_request(StationRequest::Initiate);

        if let StationResponse::Success = response {
        } else {
            assert!(matches!(response, StationResponse::Blocked));
        }

        match event.ttn {
            // enqueue handover/terminate call event
            Some(time) => {}
            // enqueue a terminate call event
            None => {
                let terminate_ev = CellEvent {
                    idx: event.idx,
                    time: event.time + event.remaining_time,
                    ty: CellEventType::TerminateCall,
                    remaining_time: 0.0,
                    ttn: None,
                    velocity: event.velocity,
                    direction: event.direction,
                    station: event.station,
                    position: todo!(),
                };
            }
        }

        todo!()
    }

    fn process_call_terminate(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::TerminateCall));
        todo!()
    }

    fn process_call_handover(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::HandoverCall));
        todo!()
    }
}
