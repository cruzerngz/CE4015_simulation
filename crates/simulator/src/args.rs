/// Main CLI arguments
#[derive(Debug, clap::Parser)]
pub struct CliArgs {
    /// Number of simulation runs to perform
    #[clap(long)]
    #[clap(default_value_t = 100)]
    pub num_runs: u32,

    /// Number of call inititation events to generate per simulation run
    #[clap(long)]
    #[clap(default_value_t = 10_000)]
    pub num_events: u32,

    /// The number of channels reserved for handover requests
    #[clap(short, long)]
    #[clap(default_value_t = 0)]
    pub reserved_handover_channels: u8,

    /// Run the simulation in antithetic pairs
    #[clap(long)]
    pub antithetic: bool,

    /// Output file for completed events in the simulation
    #[clap(long)]
    #[clap(default_value = concat!(env!("CARGO_BIN_NAME"), "_events", ".csv"))]
    pub event_log_output: String,

    /// Output file for performance measures in the simulation
    #[clap(long)]
    #[clap(default_value = concat!(env!("CARGO_BIN_NAME"), "_perf", ".csv"))]
    pub perf_measure_output: String,

    /// Common postfix for event log and performance measure output files.
    ///
    /// This postfix comes after any configured output file names and before the file extension.
    #[clap(long)]
    pub common_postfix: Option<String>,

    /// Generate a given number of call initiation events, without running the simulation.
    ///
    /// Used for verifying input modelling correctness.
    #[clap(long)]
    pub generate: Option<u32>,

    /// Generated call initiation events will be written to this path as csv.
    #[clap(long)]
    #[clap(default_value = "call_init.csv")]
    pub generate_to: String,
}
