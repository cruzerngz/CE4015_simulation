//! Core functionality for running simulations.

use std::{
    fs,
    io::{self, Write},
    ops::{Add, Div},
    path::Path,
};

pub trait EventLike {
    type SharedResources: Default;
    type EventStats;

    /// Performance measure should implement addition and division
    type PerformanceMeasure: Add<Output = Self::PerformanceMeasure> + Div<f64>;

    /// Step through one event in the simulation.
    fn step(&mut self, shared: &mut Self::SharedResources) -> Option<Vec<Self::EventStats>>;

    /// From the results of the simulation, calculate a performance measure.
    fn calculate_performance_measure(results: &[Self::EventStats]) -> Self::PerformanceMeasure;
}

/// Runner for event-based simulations
pub struct EventRunner<P>
where
    P: EventLike,
{
    inner: P,

    /// Commonly shared resources used in the simulation
    globals: P::SharedResources,

    /// The results of the simulation
    results: Vec<P::EventStats>,
    // perf_measure: Option<P::PerformanceMeasure>,
}

impl<P> EventRunner<P>
where
    P: EventLike,
{
    /// Initialize runner with the event processor.
    pub fn init(logic: P, resources: Option<P::SharedResources>) -> Self {
        Self {
            inner: logic,
            globals: resources.unwrap_or_default(),
            results: Vec::new(),
            // perf_measure: None,
        }
    }

    pub fn run(&mut self) {
        while let Some(stats) = self.inner.step(&mut self.globals) {
            self.results.extend(stats);
        }

        // self.perf_measure = Some(P::calculate_performance_measure(&self.results));
    }

    /// Returns the performance measure for the simulation run.
    pub fn performance_measure(&mut self) -> P::PerformanceMeasure {
        P::calculate_performance_measure(&self.results)
    }

    /// Write the results of the simulation as csv to a file.
    ///
    /// If set to append, headerless data is written to a file.
    pub fn write_to_file<T: AsRef<Path>>(&self, path: T, append: bool) -> io::Result<()>
    where
        P::EventStats: serde::Serialize,
    {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(!append)
            .from_writer(vec![]);

        for record in &self.results {
            writer.serialize(record)?;
        }

        writer.flush()?;

        let mut file = match append {
            // if appending, write to vec first
            true => fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?,

            // if not appending, write to file directly
            false => fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?,
        };

        file.write_all(
            &writer
                .into_inner()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        )?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
}
