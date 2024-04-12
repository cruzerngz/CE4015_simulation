/// Main CLI arguments
#[derive(Debug, clap::Parser)]
pub struct CliArgs {
    /// Number of simulation runs to perform
    #[clap(long)]
    #[clap(default_value_t = 100)]
    pub runs: u32,

    /// Number of call inititation events to generate per simulation run
    #[clap(long)]
    #[clap(default_value_t = 10_000)]
    pub events: u32,

    /// The number of channels reserved for handover requests
    #[clap(short, long)]
    #[clap(default_value_t = 0)]
    pub reserved_handover_channels: u8,

    /// Run the simulation in antithetic pairs
    #[clap(long)]
    pub antithetic: bool,

    /// Output file for the simulation results
    #[clap(long)]
    #[clap(default_value = concat!(env!("CARGO_BIN_NAME"), "_out", ".csv"))]
    pub output: String,

    /// Generate a given number of call initiation events, without running the simulation.
    ///
    /// Used for verifying input modelling correctness.
    #[clap(long)]
    pub generate: Option<u32>,

    /// Generated call initiation events will be written to this file.
    #[clap(long)]
    #[clap(default_value = "call_init.csv")]
    pub generate_to: String,
}
