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

mod args;
mod base_station;
mod event;
mod generator;
mod logic;

use std::io;

use clap::Parser;
use probability::distribution::Sample;
use probability::prelude::*;
use probability::source::Source;

fn main() -> io::Result<()> {
    let args = args::CliArgs::parse();

    println!("hello sim world!");

    let dist = probability::distribution::Uniform::new(0.0, 1.0);

    let mut source = probability::source::default(0);

    let mut total: f64 = 0.0;

    for _ in 0..1000 {
        let sample = dist.sample(&mut source);
        total += sample;
    }

    println!("average: {}", total / 1000.0);

    let mut source = source::default(42);
    let distribution = Uniform::new(0.0, 1.0);
    let sampler = Independent(&distribution, &mut source);

    // distribution.sample();
    distribution.distribution(0.0);
    // distribution.inverse(p)

    let samples = sampler.take(10).collect::<Vec<_>>();

    let mut total = 0.0;

    for _ in 0..100 {
        let val = source.read::<f64>();
        println!("val: {}", val);
        // let sampled = distribution.sample(val);
        // println!("sampled: {}", sampled);
        // total += sampled;
    }

    println!("avg: {}", total / 100.0);

    // let x = probability::distribution::Gaussian::new(mu, sigma)

    Ok(())
}
