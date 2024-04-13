#![allow(unused)]

mod args;
mod base_station;
mod event;
mod generator;
mod logic;

use clap::Parser;
use event::PerfMeasure;
use logic::{EventProcessor, Shared};
use probability::prelude::*;
use probability::source::Source;
use simulator_core::EventRunner;
use std::{fs, io};

use crate::generator::CallEventGenerator;

/// Common float type for the simulator
type FloatingPoint = f32;

/// Random number generator source
#[derive(Clone)]
struct RngSource<T>(T);

impl<T: rand::RngCore> source::Source for RngSource<T> {
    fn read_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
}

/// A deterministic source used for testing
#[derive(Clone, Debug)]
struct DetermnisticSource(u64);

impl rand::RngCore for DetermnisticSource {
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(1);
        self.0 as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // self.0 = self.0.wrapping_add(*dest.get(0).unwrap_or(&0) as u64);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        // self.0 = self.0.wrapping_add(*dest.get(0).unwrap_or(&0) as u64);
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args = args::CliArgs::parse();

    if let Some(num_gen) = args.generate {
        let generator = CallEventGenerator::new(
            1,
            RngSource(DetermnisticSource(1)),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        generate_num_to_file(generator, num_gen, &args.generate_to)?;
        return Ok(());
    }

    let shared_resources = Shared::new(args.reserved_handover_channels as usize);
    println!("base stations: {:#?}", shared_resources);

    let mut perf_measures: Vec<PerfMeasure> = Vec::new();

    for run_idx in 0..args.num_runs as usize {
        // new generator for each iteration
        let generator = CallEventGenerator::new(
            run_idx + 1,
            RngSource(rand::rngs::ThreadRng::default()),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        match args.antithetic {
            true => {
                let (events_a, events_b): (Vec<_>, Vec<_>) = generator
                    .antithetic()
                    .take(args.num_events as usize)
                    .unzip();

                let sim_a = EventProcessor::new(run_idx + 1, events_a);
                let sim_b = EventProcessor::new(run_idx + 1, events_b);

                let mut run_a = EventRunner::init(sim_a, Some(shared_resources.clone()));
                let mut run_b = EventRunner::init(sim_b, Some(shared_resources.clone()));

                run_a.run();
                run_b.run();
                let avg_perf_measure =
                    (run_a.performance_measure() + run_b.performance_measure()) / 2.0;

                perf_measures.push(avg_perf_measure);

                if run_idx == 0 {
                    run_a.write_to_file(&args.event_log_output, false)?;
                } else {
                    run_a.write_to_file(&args.event_log_output, true)?;
                }

                run_b.write_to_file(&args.event_log_output, true)?;
            }
            false => {
                let gen_events = generator.take(args.num_events as usize).collect::<Vec<_>>();

                let sim = EventProcessor::new(run_idx + 1, gen_events);
                let mut run = EventRunner::init(sim, Some(shared_resources.clone()));

                run.run();
                perf_measures.push(run.performance_measure());

                match run_idx == 0 {
                    true => run.write_to_file(&args.event_log_output, false)?,
                    false => run.write_to_file(&args.event_log_output, true)?,
                }
            }
        }
    }

    let mut writer = csv::Writer::from_path(&args.perf_measure_output)?;
    for perf in perf_measures {
        writer.serialize(perf)?;
    }

    writer.flush()?;

    Ok(())
}

fn generate_num_to_file<S>(
    event_gen: CallEventGenerator<S>,
    num_gen: u32,
    file: &str,
) -> io::Result<()>
where
    S: Source + Clone,
{
    let mut writer = csv::Writer::from_path(file)?;

    for ev in event_gen.take(num_gen as usize) {
        // println!("event time: {}", ev.time);
        writer.serialize(ev)?;
    }

    writer.flush()?;

    Ok(())
}
