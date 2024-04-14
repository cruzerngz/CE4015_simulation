// #![allow(unused)]

mod args;
mod base_station;
mod event;
mod generator;
mod logic;

use clap::Parser;
use event::{CellEventResult, PerfMeasure};
use logic::{EventProcessor, Shared};
use probability::prelude::*;
use probability::source::Source;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use simulator_core::EventRunner;
use std::{
    io,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::generator::CallEventGenerator;

/// Common float type for the simulator
type FloatingPoint = f64;

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

    fn fill_bytes(&mut self, _: &mut [u8]) {
        // self.0 = self.0.wrapping_add(*dest.get(0).unwrap_or(&0) as u64);
    }

    fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), rand::Error> {
        // self.0 = self.0.wrapping_add(*dest.get(0).unwrap_or(&0) as u64);
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args = args::CliArgs::parse();

    let (event_log_path, perf_measure_path) = match args.common_postfix {
        Some(post) => {
            let mut ev_path = PathBuf::from(&args.event_log_output);
            let mut perf_path = PathBuf::from(&args.perf_measure_output);

            let pre = ev_path.file_stem();
            let ext = ev_path.extension();
            let pre = pre.and_then(|p| p.to_str()).unwrap_or("");
            let mut appended = format!("{}_{}", pre, post);
            if let Some(ext) = ext {
                appended = format!("{}.{}", appended, ext.to_str().unwrap());
            }
            ev_path.set_file_name(appended);

            let pre = perf_path.file_stem();
            let ext = perf_path.extension();
            let pre = pre.and_then(|p| p.to_str()).unwrap_or("");
            let mut appended = format!("{}_{}", pre, post);
            if let Some(ext) = ext {
                appended = format!("{}.{}", appended, ext.to_str().unwrap());
            }
            perf_path.set_file_name(appended);

            (ev_path, perf_path)
        }
        None => (
            PathBuf::from(&args.event_log_output),
            PathBuf::from(&args.perf_measure_output),
        ),
    };

    // let (handle, send_chan) = csv_writer_task(event_log_path.clone());

    // println!("event log path: {:#?}", event_log_path);
    // println!("perf measure path: {:#?}", perf_measure_path);

    if let Some(num_gen) = args.generate {
        let generator = CallEventGenerator::new(
            1,
            RngSource(rand::rngs::OsRng),
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
    // debug_println!("base stations: {:#?}", shared_resources);

    let perf_measures: Arc<Mutex<Vec<PerfMeasure>>> = Arc::new(Mutex::new(Vec::new()));

    (0..args.num_runs as usize)
        .into_par_iter()
        .for_each(|run_idx| {
            // println!("#{} starting run", run_idx);

            // new generator for each iteration
            let generator = CallEventGenerator::new(
                run_idx + 1,
                RngSource(rand::rngs::OsRng),
                None,
                None,
                None,
                None,
                None,
                None,
            );

            match args.antithetic {
                true => {
                    // println!("#{} generating antithetic events", run_idx);
                    let (events_a, events_b): (Vec<_>, Vec<_>) = generator
                        .antithetic()
                        .take(args.num_events as usize)
                        .unzip();

                    let sim_a = EventProcessor::new(run_idx + 1, events_a);
                    let sim_b = EventProcessor::new(run_idx + 1, events_b);

                    let mut run_a = EventRunner::init(sim_a, Some(shared_resources.clone()));
                    let mut run_b = EventRunner::init(sim_b, Some(shared_resources.clone()));

                    // println!("#{} starting simulation", run_idx);

                    run_a.run();
                    run_b.run();

                    // println!(
                    //     "#{} simulation complete, calculating perf measure ",
                    //     run_idx
                    // );
                    let avg_perf_measure =
                        (run_a.performance_measure() + run_b.performance_measure()) / 2.0;

                    perf_measures.lock().unwrap().push(avg_perf_measure);

                    if args.skip_event_log {
                        return;
                    }

                    if run_idx == 0 {
                        run_a
                            .write_to_file(&event_log_path, false)
                            .expect("failed to write to file");
                    } else {
                        run_a
                            .write_to_file(&event_log_path, true)
                            .expect("failed to write to file");
                    }

                    run_b
                        .write_to_file(&event_log_path, true)
                        .expect("failed to write to file");
                }
                false => {
                    let gen_events = generator.take(args.num_events as usize).collect::<Vec<_>>();

                    let sim = EventProcessor::new(run_idx + 1, gen_events);
                    let mut run = EventRunner::init(sim, Some(shared_resources.clone()));

                    run.run();
                    perf_measures
                        .lock()
                        .unwrap()
                        .push(run.performance_measure());

                    if args.skip_event_log {
                        return;
                    }

                    match run_idx == 0 {
                        true => run
                            .write_to_file(&event_log_path, false)
                            .expect("failed to write to file"),
                        false => run
                            .write_to_file(&event_log_path, true)
                            .expect("failed to write to file"),
                    }
                }
            }

            // println!("#{} ending run", run_idx);
        });

    // drop(send_chan);
    // handle.join().unwrap();

    let mut writer = csv::Writer::from_path(&perf_measure_path)?;
    for perf in perf_measures.lock().unwrap().iter() {
        writer.serialize(perf)?;
    }

    writer.flush()?;

    Ok(())
}

/// Run the csv writer in a separate task
///
/// Turns out funneling all write ops to a single thread is slow
#[allow(unused)]
fn csv_writer_task<T: Into<PathBuf> + Send + 'static>(
    path: T,
) -> (
    thread::JoinHandle<()>,
    mpsc::SyncSender<Vec<CellEventResult>>,
) {
    let (send, recv) = mpsc::sync_channel(10);

    let handle = thread::spawn(move || {
        let mut writer = csv::Writer::from_path(path.into()).expect("failed to open file");
        // let mut count = 0;

        while let Ok(data) = recv.recv() {
            // count += 1;
            // println!("#{} run received", count);

            for ev in data {
                writer.serialize(ev).expect("failed to write to file");
            }
        }

        writer.flush().expect("failed to flush csv writer");
    });

    (handle, send)
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

/// Prints as per normal when in debug mode.
/// Does not print when in release mode.
#[macro_export]
macro_rules! debug_println {
    ($($args:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($args)*);
    };
}

#[macro_export]
macro_rules! debug_print {
    ($($args:tt)*) => {
        #[cfg(debug_assertions)]
        print!($($args)*);
    };
}

// use debug_println;
// use debug_print;
