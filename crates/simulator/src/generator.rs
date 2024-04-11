//! Random variable generators for various parts of the sim

use std::path::Iter;

use probability::{
    distribution::{Inverse, Sample},
    sampler::Independent,
    source::{self, Source},
};

/// A generator that generates random variables from some inner distribution.
#[derive(Clone, Debug)]

pub struct SingleVariateIterator<D, S>
where
    D: Sample,
    S: Source,
{
    source: S,
    distribution: D,
    // sampler: Independent<D, &'s mut S>,
}

#[derive(Clone, Debug)]
pub struct AntitheticIterator<D, S>
where
    D: Sample,
    S: Source,
{
    source: S,
    distribution: D,
}

/// An anththetic sampler that yields 2 samples from a reference sampler.
///
/// Any further samples will return the same value as the second sample.
#[derive(Debug)]
struct AntitheticSampler<'s, S>
where
    S: Source,
{
    source: &'s mut S,
    first_drawn: Option<u64>,
}

impl<'s, S> AntitheticSampler<'s, S>
where
    // D: Sample + Inverse,
    S: Source,
{
    pub fn new(source: &'s mut S) -> Self {
        Self {
            source,
            first_drawn: None,
        }
    }
}

impl<'s, S> Source for AntitheticSampler<'s, S>
where
    S: Source,
{
    fn read_u64(&mut self) -> u64 {
        match self.first_drawn {
            Some(first) => u64::MAX - first,
            None => {
                let first = self.source.read_u64();
                self.first_drawn = Some(first);
                first
            }
        }
    }
}

impl<D, S> SingleVariateIterator<D, S>
where
    D: Sample + Clone,
    S: Source + Clone,
{
    pub fn new(distribution: D, mut source: S) -> Self {
        Self {
            source,
            distribution,
        }
    }

    /// Create a new iterator that generates antithetic pairs from the distribution.
    pub fn antithetic_iter(&self) -> AntitheticIterator<D, S> {
        AntitheticIterator {
            source: self.source.clone(),
            distribution: self.distribution.clone(),
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
    D: Sample + Inverse,
    S: Source + Clone,
{
    type Item = (D::Value, D::Value);

    fn next(&mut self) -> Option<Self::Item> {
        let mut anti_sampler = AntitheticSampler::new(&mut self.source);

        let sample_a = self.distribution.sample(&mut anti_sampler);
        let sample_b = self.distribution.sample(&mut anti_sampler);

        Some((sample_a, sample_b))
    }
}

#[cfg(test)]
mod tests {

    use probability::distribution::Uniform;

    use super::*;

    #[test]
    fn test_antithetic_iter() {
        let mut gen = SingleVariateIterator::new(Uniform::new(0.0, 10.0), source::default(0));

        for sample in gen.clone().take(10) {
            println!("{:?}", sample);
        }

        let sum = gen.clone().take(1000).sum::<f64>();
        let avg = sum / 1000.0;
        println!("average: {}", avg);

        let antithetic = gen.antithetic_iter();

        // for uniform anthithetic pairs, the average between pairs should be around 5
        let sum = antithetic
            .clone()
            .take(1000)
            .map(|(a, b)| (a + b) / 2.0)
            .sum::<f64>();

        let avg = sum / 1000.0;
        println!("average: {}", avg);

        let v = antithetic
            .clone()
            .take(10)
            //     .map(|(a, b)| (a + b) / 2.0)
            .collect::<Vec<_>>();

        println!("{:#?}", v);
    }
}
