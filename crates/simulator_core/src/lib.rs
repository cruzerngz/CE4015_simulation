//! Core functionality for running simulations.

use std::{
    fs,
    io::{self, Write},
};

pub trait EventLike {
    type SharedResources: Default;
    type EventStats;

    /// Step through one event in the simulation.
    fn step(&mut self, shared: &mut Self::SharedResources) -> Option<Vec<Self::EventStats>>;
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
        }
    }

    pub fn run(&mut self) {
        while let Some(stats) = self.inner.step(&mut self.globals) {
            self.results.extend(stats);
        }
    }

    /// Write the results of the simulation as csv to a file.
    ///
    /// If set to append, headerless data is written to a file.
    pub fn write_to_file(&self, path: &str, append: bool) -> io::Result<()>
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
                .expect("failed to extract inner buffer from csv writer"),
        )?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
}
