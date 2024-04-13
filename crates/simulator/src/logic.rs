//! Event processing logic

use std::collections::VecDeque;

use simulator_core::EventLike;

use crate::{
    base_station::{BaseStation, StationRequest, StationResponse},
    event::{
        BaseStationIdx, CellEvent, CellEventResult, CellEventType, PerfMeasure,
        RelativeVehiclePosition, VehicleDirection,
    },
    generator::{calculate_ttn, VEHICLE_LOC_DIST},
};

/// Process events in the simulation
#[derive(Debug)]
pub struct EventProcessor {
    run_num: usize,
    /// Future events to process
    fel: VecDeque<CellEvent>,
}

/// Shared resources in the simulation
#[derive(Clone, Debug, Default)]
pub struct Shared {
    /// Base stations in the simulation
    base_stations: [BaseStation; 20],
}

impl EventLike for EventProcessor {
    type SharedResources = Shared;

    type EventStats = CellEventResult;

    type PerformanceMeasure = PerfMeasure;

    fn step(&mut self, shared: &mut Self::SharedResources) -> Option<Vec<Self::EventStats>> {
        let next_event = self.fel.pop_front()?;

        let results = match next_event.ty {
            crate::event::CellEventType::Initiate => {
                self.process_call_initiation(next_event, shared)
            }
            crate::event::CellEventType::Terminate => {
                self.process_call_terminate(next_event, shared)
            }
            crate::event::CellEventType::Handover => self.process_call_handover(next_event, shared),
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
        assert!(matches!(event.ty, CellEventType::Initiate));

        // check shared resource

        let station = shared
            .base_stations
            .get_mut(event.station as usize)
            .expect("station must exist");

        let response = station.process_request(StationRequest::Initiate);

        let ev_result = CellEventResult {
            idx: event.idx,
            run: event.run,
            time: event.time,
            ty: event.ty,
            outcome: response,
            direction: event.direction,
            speed: event.velocity,
            station: event.station,
        };

        let mut results = vec![ev_result];

        if let StationResponse::Blocked = response {
            return results;
        }

        match event.ttn {
            // enqueue handover/terminate call event
            Some(tt_next) => {
                let remaining_call_time = event.remaining_time - tt_next;

                // check for termination
                // Time not correct
                let next_ev = match (event.station, event.direction) {
                    (BaseStationIdx::One, VehicleDirection::EastToWest)
                    | (BaseStationIdx::Twenty, VehicleDirection::WestToEast) => CellEvent {
                        idx: event.idx,
                        run: event.run,
                        time: event.time + tt_next,
                        ty: CellEventType::Terminate,
                        remaining_time: remaining_call_time,
                        ttn: None,
                        velocity: event.velocity,
                        direction: event.direction,
                        station: event.station,
                        position: match event.direction {
                            VehicleDirection::EastToWest => RelativeVehiclePosition::WestEnd,
                            VehicleDirection::WestToEast => RelativeVehiclePosition::EastEnd,
                        },
                    },

                    _ => CellEvent {
                        idx: event.idx,
                        run: event.run,
                        time: event.time + tt_next,
                        ty: CellEventType::Handover,
                        remaining_time: remaining_call_time,
                        ttn: calculate_ttn(
                            remaining_call_time,
                            0.0,
                            event.velocity,
                            event.direction,
                        ),
                        velocity: event.velocity,
                        direction: event.direction,
                        station: event.station,
                        position: match event.direction {
                            VehicleDirection::EastToWest => RelativeVehiclePosition::WestEnd,
                            VehicleDirection::WestToEast => RelativeVehiclePosition::EastEnd,
                        },
                    },
                };

                self.insert_event(next_ev);
            }
            // enqueue a terminate call event
            None => {
                let terminate_ev = CellEvent {
                    idx: event.idx,
                    run: event.run,
                    time: event.time + event.remaining_time,
                    ty: CellEventType::Terminate,
                    remaining_time: 0.0,
                    ttn: None,
                    velocity: event.velocity,
                    direction: event.direction,
                    station: event.station,
                    position: {
                        let dist = event.velocity / 3.6 * event.remaining_time;
                        let pos = match event.direction {
                            VehicleDirection::EastToWest => {
                                RelativeVehiclePosition::Other(event.position.to_float() - dist)
                            }
                            VehicleDirection::WestToEast => {
                                RelativeVehiclePosition::Other(event.position.to_float() + dist)
                            }
                        };

                        assert!(
                            pos.to_float() >= VEHICLE_LOC_DIST.0,
                            "vehicle position must be within station bounds"
                        );
                        assert!(
                            pos.to_float() <= VEHICLE_LOC_DIST.1,
                            "vehicle position must be within station bounds"
                        );

                        pos
                    },
                };

                // TODO: handle event immediately if future event occurrs at the same time
                self.insert_event(terminate_ev);
            }
        }

        results
    }

    fn process_call_terminate(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::Terminate));

        let station = shared
            .base_stations
            .get_mut(event.station as usize)
            .expect("station must exist");

        let res = station.process_request(StationRequest::Terminate);

        let result = event.to_result(res);

        vec![result]
    }

    fn process_call_handover(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::Handover));

        let depart_station = shared
            .base_stations
            .get_mut(event.station as usize)
            .expect("station must exist");

        let arr_station = shared
            .base_stations
            .get_mut(match event.direction {
                VehicleDirection::EastToWest => event.station as usize - 1,
                VehicleDirection::WestToEast => event.station as usize + 1,
            })
            .expect("station must exist");

        todo!()
    }
}
