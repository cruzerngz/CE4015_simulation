/// Main CLI arguments
#[derive(Debug, clap::Parser)]
pub struct CliArgs {
    /// Number of simulation runs to perform
    #[clap(long)]
    #[clap(default_value_t = 100)]
    pub runs: u32,

    /// The number of channels reserved for handover requests
    #[clap(short, long)]
    #[clap(default_value_t = 0)]
    pub reserved_handover_channels: u8,

    /// Run the simulation in antithetic pairs
    pub antithetic: bool,

}
