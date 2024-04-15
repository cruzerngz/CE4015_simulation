//! Core functionality for running event-based simulations.

use std::{
    fs,
    io::{self, Write},
    ops::{Add, Div},
    path::Path,
};

use probability::{
    distribution::{self, Distribution, Sample},
    source::Source,
};

/// Event processing logic implements this trait.
pub trait EventLike {
    /// Shared resources used in the simulation, such as statistical counters, etc.
    type SharedResources: Default;

    /// Statistics returned after every event in the simulation.
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

/// A generator that generates random variables from some inner distribution.
#[derive(Clone, Debug)]
pub struct SingleVariateIterator<D, S>
where
    D: Sample,
    S: Source,
{
    source: S,
    distribution: D,
}

#[derive(Clone, Debug)]
pub struct AntitheticIterator<D, S>
where
    D: Sample,
    S: Source,
{
    source: S,
    distribution: D,
    prepare: usize,
}

/// An anththetic sampler that can yield 10 antithetic samples from a reference sampler.
///
/// Any further samples will return the same value as the second sample.
#[derive(Debug)]
pub struct AntitheticSampler<'s, S>
where
    S: Source,
{
    source: &'s mut S,
    // first_drawn: Option<u64>,
    /// Current index of the sample cache
    cached: Option<usize>,
    drain: bool,
    sample_store: Vec<u64>,
}

/// Exponential distribution with a location parameter.
#[derive(Clone, Debug)]
pub struct ExponentialLoc {
    inner: distribution::Exponential,
    loc: f64,
}

impl Distribution for ExponentialLoc {
    type Value = f64;

    fn distribution(&self, x: f64) -> f64 {
        self.inner.distribution(x - self.loc)
    }
}

impl Sample for ExponentialLoc {
    fn sample<S>(&self, source: &mut S) -> Self::Value
    where
        S: Source,
    {
        self.inner.sample(source) + self.loc
    }
}

impl ExponentialLoc {
    pub fn new(lambda: f64, loc: f64) -> Self {
        Self {
            inner: distribution::Exponential::new(lambda),
            loc,
        }
    }
}

impl<'s, S> AntitheticSampler<'s, S>
where
    S: Source,
{
    pub fn new(source: &'s mut S) -> Self {
        Self {
            source,
            cached: None,
            drain: false,
            sample_store: Vec::new(),
        }
    }

    /// Pre-generate samples from the source
    pub fn prepare(&mut self, num: usize) {
        self.cached = Some(0);
        for _ in 0..num {
            self.sample_store.push(self.source.read_u64());
        }
    }

    // /// Accumulate and return samples from the source
    // pub fn accumulate(&mut self) {
    //     self.drain = false;
    // }

    /// Take samples that have been accumulated instead of the source
    pub fn drain(&mut self) {
        self.drain = true;
    }
}

impl<'s, S> Source for AntitheticSampler<'s, S>
where
    S: Source,
{
    fn read_u64(&mut self) -> u64 {
        match (self.drain, &mut self.cached) {
            (true, _) => {
                // debug_println!("reading from store");
                match self.sample_store.len() {
                    0 => 0,
                    1 => u64::MAX - self.sample_store[0],
                    _ => u64::MAX - self.sample_store.remove(0),
                }
            }
            (false, None) => {
                // debug_println!("reading from source");
                let sample = self.source.read_u64();
                self.sample_store.push(sample);
                sample
            }
            (false, Some(pos)) => {
                // debug_println!("reading from cache");
                match self.sample_store.len() > *pos {
                    true => {
                        let sample = self.sample_store[*pos];
                        *pos += 1;
                        sample
                    }
                    false => self.sample_store[self.sample_store.len() - 1],
                }
            }
        }
    }
}

impl<D, S> SingleVariateIterator<D, S>
where
    D: Sample + Clone,
    S: Source + Clone,
{
    pub fn new(distribution: D, source: S) -> Self {
        Self {
            source,
            distribution,
        }
    }

    /// Create a new iterator that generates antithetic pairs from the distribution.
    ///
    /// The `prepared` parameter specifies the number of samples to pre-generate.
    /// This is should be greater than the maximum number of samples that are used
    /// to generate a random variable.
    ///
    /// 10 is a good number to start with.
    pub fn antithetic_iter(&self, prepared: usize) -> AntitheticIterator<D, S> {
        AntitheticIterator {
            source: self.source.clone(),
            distribution: self.distribution.clone(),
            prepare: prepared,
        }
    }
}

impl<D, S> Iterator for SingleVariateIterator<D, S>
where
    D: Sample,
    S: Source,
{
    type Item = D::Value;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.distribution.sample(&mut self.source))
    }
}

impl<D, S> Iterator for AntitheticIterator<D, S>
where
    D: Sample,
    S: Source,
{
    type Item = (D::Value, D::Value);

    fn next(&mut self) -> Option<Self::Item> {
        // debug_println!("creating antithetic sampler");
        let mut anti_sampler = AntitheticSampler::new(&mut self.source);

        anti_sampler.prepare(self.prepare);
        // debug_println!("sampling A");
        let sample_a = self.distribution.sample(&mut anti_sampler);

        anti_sampler.drain();
        // debug_println!("sampling B");
        let sample_b: <D as Distribution>::Value = self.distribution.sample(&mut anti_sampler);

        // debug_println!("returning samples");
        Some((sample_a, sample_b))
    }
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
    ///
    /// Skips the first set of events as warmup.
    pub fn performance_measure(&mut self, skip: usize) -> P::PerformanceMeasure {
        P::calculate_performance_measure(&self.results[skip..])
    }

    /// Returns the results of the simulation run, comsuming the runner.
    pub fn into_results(self) -> Vec<P::EventStats> {
        self.results
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
