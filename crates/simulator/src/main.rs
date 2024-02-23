#![allow(unused)]

// use traits::{EventLike, Process, SimExecutor, ToEventIterator};
// mod traits;

// #[derive(Clone, Copy)]
// struct TrainSimulation;

// impl EventLike for TrainSimulation {
//     fn next_event(&mut self) -> traits::SimStatus {
//         traits::SimStatus::Continue
//     }
// }

// fn main() {
//     let sim = TrainSimulation;

//     let mut runner: SimExecutor<Process, traits::EventIterator<TrainSimulation>> =
//         SimExecutor::from_iterator(sim.to_event_iter());

//     runner.execute();
// }


mod event;

fn main() {
    println!("hello sim world!")
}
