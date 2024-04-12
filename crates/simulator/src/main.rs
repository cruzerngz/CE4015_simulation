mod args;
mod base_station;
mod event;
mod generator;
mod logic;

use clap::Parser;
use logic::{EventProcessor, Shared};
use probability::prelude::*;
use probability::source::Source;
use simulator_core::EventRunner;
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

    match args.generate {
        Some(num_gen) => {
            let generator = CallEventGenerator::new(
                RngSource(rand::rngs::ThreadRng::default()),
                None,
                None,
                None,
                None,
                None,
            );

            generate_num_to_file(generator, num_gen, &args.generate_to)?;
            return Ok(());
        }
        None => (),
    }

    let shared_resources = Shared::new(args.reserved_handover_channels as usize);
    for iteration in 0..args.runs {
        let generator = CallEventGenerator::new(
            RngSource(rand::rngs::ThreadRng::default()),
            None,
            None,
            None,
            None,
            None,
        );

        match args.antithetic {
            true => {
                let (events_a, events_b): (Vec<_>, Vec<_>) =
                    generator.antithetic().take(100).unzip();

                let sim_a = EventProcessor::new(events_a);
                let sim_b = EventProcessor::new(events_b);

                let mut run_a = EventRunner::init(sim_a, Some(shared_resources.clone()));
                let mut run_b = EventRunner::init(sim_b, Some(shared_resources.clone()));

                run_a.run();
                run_b.run();

                if iteration == 0 {
                    run_a.write_to_file(&args.output, false)?;
                } else {
                    run_a.write_to_file(&args.output, true)?;
                }

                run_b.write_to_file(&args.output, true)?;
            }
            false => {
                let gen_events = generator.take(100).collect::<Vec<_>>();

                let sim = EventProcessor::new(gen_events);
                let mut run = EventRunner::init(sim, Some(shared_resources.clone()));

                run.run();

                match iteration == 0 {
                    true => run.write_to_file(&args.output, false)?,
                    false => run.write_to_file(&args.output, true)?,
                }
            }
        }

        // let logic = EventProcessor::new(events)
    }

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
        writer.serialize(ev)?;
    }

    writer.flush()?;

    Ok(())
}
