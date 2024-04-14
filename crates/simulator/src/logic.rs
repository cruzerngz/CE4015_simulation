//! Event processing logic

use std::{collections::VecDeque, fmt::Display};

use simulator_core::EventLike;

use crate::{
    base_station::{BaseStation, StationRequest, StationResponse},
    debug_println,
    event::{
        CellEvent, CellEventResult, CellEventType, PerfMeasure, RelativeVehiclePosition,
        VehicleDirection,
    },
    generator::{calculate_ttn, VEHICLE_LOC_DIST},
    FloatingPoint,
};

/// Process events in the simulation
#[derive(Debug)]
pub struct EventProcessor {
    #[allow(dead_code)]
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

        debug_println!(
            "\nevent {}: {:?} at station {:?}, dir {:?}",
            next_event.idx,
            next_event.ty,
            next_event.station,
            next_event.direction
        );
        debug_println!("event time: {}", next_event.time);
        debug_println!("event remaining time: {}", next_event.remaining_time);
        debug_println!(
            "velocity: {} km/h covers 2000m in {}s",
            next_event.velocity,
            2000.0 / next_event.velocity * 3.6
        );
        debug_println!("{}", shared);

        let results = match next_event.ty {
            CellEventType::Initiate => self.process_call_initiation(next_event, shared),
            CellEventType::Terminate => self.process_call_terminate(next_event, shared),
            CellEventType::Handover => self.process_call_handover(next_event, shared),
        };

        Some(results)
    }

    fn calculate_performance_measure(results: &[Self::EventStats]) -> Self::PerformanceMeasure {
        let num_initiated_calls = results.iter().map(|res| res.idx).max().unwrap_or_default();

        let num_blocked_calls: usize = results
            .iter()
            .map(|res| match res.outcome {
                StationResponse::Blocked => 1_usize,
                _ => 0,
            })
            .sum();

        let num_terminated_calls: usize = results
            .iter()
            .map(|res| match res.outcome {
                StationResponse::Terminated => 1_usize,
                _ => 0,
            })
            .sum();

        PerfMeasure {
            blocked_calls: num_blocked_calls as FloatingPoint
                / num_initiated_calls as FloatingPoint,
            dropped_calls: num_terminated_calls as FloatingPoint
                / num_initiated_calls as FloatingPoint,
        }
    }
}

impl Display for Shared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let station_arr = self
            .base_stations
            .iter()
            .enumerate()
            .map(|(idx, s)| format!("{:02}::{:02}", idx + 1, s.available_channels))
            .collect::<Vec<_>>()
            .join("|");

        write!(f, "{}", station_arr)
    }
}

impl Shared {
    pub fn new(handover_reserve: usize) -> Self {
        Self {
            base_stations: {
                core::array::from_fn(|_idx| BaseStation::new(10, Some(handover_reserve)))
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
        debug_println!("inserting {:?} event into fel", event.ty);

        match self.fel.binary_search(&event) {
            Ok(pos) | Err(pos) => {
                self.fel.insert(pos, event);
            }
        }
    }

    /// Logic for creating future handover/termination events after an initiation/handover
    fn handle_handover_terminate(&mut self, event: CellEvent) -> Vec<CellEventResult> {
        match event.ttn {
            // enqueue handover/terminate call event
            Some(tt_next) => {
                let remaining_call_time = event.remaining_time - tt_next;

                let next_ev = match event.station.next_station(event.direction) {
                    // next station exists, enqueue handover event
                    Some(next_station) => CellEvent {
                        idx: event.idx,
                        run: event.run,
                        time: event.time + tt_next,
                        ty: CellEventType::Handover,
                        remaining_time: remaining_call_time,
                        ttn: calculate_ttn(
                            remaining_call_time,
                            event.position.to_float(),
                            event.velocity,
                            event.direction,
                        ),
                        velocity: event.velocity,
                        direction: event.direction,
                        // handover station refers to the station that the vehicle will connect to
                        station: next_station,
                        // position is relative to the new handover station!
                        position: match event.direction {
                            VehicleDirection::EastToWest => RelativeVehiclePosition::EastEnd,
                            VehicleDirection::WestToEast => RelativeVehiclePosition::WestEnd,
                        },
                    },
                    // next station does not exist, enqueue terminate event
                    None => CellEvent {
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
                };

                self.insert_event(next_ev);
            }

            // call about to end, terminate
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
                        debug_println!("dist: {}", dist);
                        let pos = match event.direction {
                            VehicleDirection::EastToWest => {
                                RelativeVehiclePosition::Other(event.position.to_float() - dist)
                            }
                            VehicleDirection::WestToEast => {
                                RelativeVehiclePosition::Other(event.position.to_float() + dist)
                            }
                        };

                        debug_println!("vehicle position: {:?}", pos);
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

        vec![]
    }

    // event parameter must be a call initiation event
    fn process_call_initiation(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::Initiate));

        // check shared resource

        let station = &mut shared.base_stations[event.station as usize];

        let response = station.process_request(StationRequest::Initiate, event.idx);
        debug_println!("call init response: {:?}", response);

        let ev_result = event.to_result(response, station.available_channels);

        let mut results = vec![ev_result];

        if let StationResponse::Blocked = response {
            return results;
        }

        let additional_res = self.handle_handover_terminate(event);
        results.extend(additional_res);

        results
    }

    fn process_call_terminate(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::Terminate));

        let station = &mut shared.base_stations[event.station as usize];

        let res = station.process_request(StationRequest::Terminate, event.idx);
        assert!(matches!(res, StationResponse::Success));

        let result = event.to_result(res, station.available_channels);

        vec![result]
    }

    fn process_call_handover(
        &mut self,
        event: CellEvent,
        shared: &mut Shared,
    ) -> Vec<CellEventResult> {
        assert!(matches!(event.ty, CellEventType::Handover));

        let prev_idx = event.station.previous_station(event.direction).unwrap();
        // debug_println!("previous station: {:?}", prev_idx);

        let depart_station = &mut shared.base_stations[prev_idx as usize];

        debug_println!("attempting disconnect from station {:?}", prev_idx);
        let res = depart_station.process_request(StationRequest::HandoverDisconnect, event.idx);
        assert!(matches!(res, StationResponse::Success));

        let arr_station = &mut shared.base_stations[event.station as usize];

        let res = arr_station.process_request(StationRequest::HandoverConnect, event.idx);

        let mut results = vec![event.to_result(res, arr_station.available_channels)];

        // u are failure
        if let StationResponse::Terminated = res {
            return results;
        }

        let additional_res = self.handle_handover_terminate(event);
        results.extend(additional_res);

        results
    }
}

#[cfg(test)]
mod tests {
    use crate::{event::BaseStationIdx, FloatingPoint};

    use super::*;

    #[test]
    fn test_shared_display() {
        let shared = Shared::new(1);
        debug_println!("{}", shared);
    }

    #[test]
    fn test_insert_into_fel() {
        let events = (0..10).into_iter().map(|idx| CellEvent {
            idx,
            run: 0,
            time: idx as FloatingPoint,
            ty: CellEventType::Initiate,
            remaining_time: 0.0,
            ttn: None,
            velocity: 0.0,
            direction: VehicleDirection::EastToWest,
            station: BaseStationIdx::One,
            position: RelativeVehiclePosition::EastEnd,
        });

        let mut proc = EventProcessor::new(1, vec![]);
        for ev in events {
            proc.insert_event(ev);
        }

        debug_println!("fel len: {}", proc.fel.len());
        debug_println!("{:#?}", proc.fel);
    }
}
