# CE4015_simulation
Course project for CE4015 Simulation and Modelling

## Overview
- Logic located in [simulator](./crates/simulator/)
- Variate generators, sim runners located in [simulator_core](./crates/simulator_core/)
- Input/output analysis located in [analysis](./analysis/)

## Build
```sh
cargo build --release # build
cargo run --release -- --help # view help
```

## Performance
For event-based simulations running using naive event generation,
the simulator can step through 1,000,000,000 initial events (~8B processed events) in approximately 3 minutes on a fully loaded 12-core CPU.

For event-based simulations that are generated in antithetic variate (AV) pairs,
the same simulation takes approximately 10 times longer.

For this project, the slowdown is due to pre-generating 10 random samples for every AV variable and effectively increasing the compute workload by an order of magnitude.

By fine-tuning the amount of pregenerated samples, the simulation can be made to complete more quickly.

## Output
Note that event logs generate a substantial amount of data. 10,000 iterations of 10,000 call initiation events (800M processed) generated approximately 58GB of data.

If performance measure is the only result required from a simulation, run the simulation with `--skip-event-log`.
If the event logs are required, the simulator should be run with only a few iterations.
