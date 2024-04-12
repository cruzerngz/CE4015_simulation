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

use clap::Parser;
use core::num;
use probability::distribution::Sample;
use probability::prelude::*;
use probability::source::Source;
use std::io;

use crate::generator::CallEventGenerator;

/// Random number generator source
#[derive(Clone)]
struct RngSource<T>(T);

impl<T: rand::RngCore> source::Source for RngSource<T> {
    fn read_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
}

fn main() -> io::Result<()> {
    let args = args::CliArgs::parse();

    let generator = CallEventGenerator::new(
        RngSource(rand::rngs::ThreadRng::default()),
        None,
        None,
        None,
        None,
        None,
    );

    match args.generate {
        Some(num_gen) => {
            generate_num_to_file(generator, num_gen, &args.generate_to)?;
        }
        None => todo!(),
    }

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

    // probability::distribution::Discrete
    // let x = probability::distribution::Gaussian::new(mu, sigma)

    Ok(())
}

fn generate_num_to_file<S>(
    mut event_gen: CallEventGenerator<S>,
    num_gen: u32,
    file: &str,
) -> io::Result<()>
where
    S: Source + Clone,
{
    let mut source = probability::source::default(0);

    let mut writer = csv::Writer::from_path(file)?;

    for ev in event_gen.take(num_gen as usize) {
        writer.serialize(ev)?;
    }

    writer.flush()?;

    Ok(())
}
