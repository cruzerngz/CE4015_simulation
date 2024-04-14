
// nice code formatting
#show raw.where(lang: "rs"): code => {
  block(
    fill: luma(247),
    radius: 3pt,
    outset: 5pt,
    stroke: gray,
    breakable: true,
    width: 100%, // does this look nice?
  )[#text(size: 0.8em)[#code]]
}

// table style - I want to emulate the look seen in papers
#set table(
  stroke: (x, y) => (
    y: if y <= 1 {1pt} else {0.1pt},
    left: 0pt,
    right: 0pt,
    bottom: 1pt,
  ),
)

// cover page
#align(center, text(size: 1.5em, weight: "bold")[

  #image("media/ntu_logo.svg", width: 75%)

  CE4015: Simulation and Modelling
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  2023/2024 Semester 2 Assignment 2 part 2:

  Implementation of a discrete event simulator
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  Ng Jia Rui: U2020777D
  #linebreak()
  #linebreak()
  SCHOOL OF COMPUTER SCIENCE AND ENGINEERING

  NANYANG TECHNOLOGICAL UNIVERSITY

  #pagebreak()
])

#set page(numbering: "1")

// outline
#outline(indent: true,)
#linebreak()
#outline(title: "Tables", target: figure.where(kind: table))
#linebreak()
#outline(title: "Figures", target: figure.where(kind: image))
#pagebreak()

#set heading(numbering: "1.1.")

= Overview
In this report, the discrete event simulator in part 1 is implemented as a terminating simulation.

= Design revision
This simulation now uses antithetic variables (AV) to reduce the variance of the simulation results.

Each event in a simulation run is generated in pairs, by using the same random seed (0.0 - 1.0) and taking the inverse for the second item in the pair.

Due to the way the probability distribution function for normal distributions is implemented, there is an arbitrary number of calls to the random sampler during the generation of a single random normal sample @JSSv005i08.

In this simulation experiment, the random sampler is initialized with a 10-element buffer of `u64` to ensure that the distribution function receives the same but inverted set of random samples.

```rs
/// An antithetic sampler that can yield a specified maximum number of antithetic samples from a reference sampler.
///
struct AntitheticSampler<'s, S: Source> {
    source: &'s mut S,
    /// Current index of the sample cache
    cached: Option<usize>,
    drain: bool,
    sample_store: Vec<u64>,
}

impl<'s, S: Source> AntitheticSampler<'s, S> {
  /// Pre-generate samples from the source
  pub fn prepare(&mut self, num: usize) {
      self.cached = Some(0);
      for _ in 0..num {
          self.sample_store.push(self.source.read_u64());
      }
  }
}

impl<'s, S> Source for AntitheticSampler<'s, S: Source> {
    fn read_u64(&mut self) -> u64 {
        match (self.drain, &mut self.cached) {
            (true, _) => {
                match self.sample_store.len() {
                    0 => 0,
                    1 => u64::MAX - self.sample_store[0],
                    _ => u64::MAX - self.sample_store.remove(0),
                }
            }
            (false, None) => {
                let sample = self.source.read_u64();
                self.sample_store.push(sample);
                sample
            }
            (false, Some(pos)) => {
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
```

Event handling has been updated slightly. When processing an event and encountering a failed channel allocation, a termination event is no longer generated.

The terminating condition is now based on the number of call initiations, rather than checking the FEL for any event.

= Input modelling
The following distributions are determined from the sample file `PCS_TEST_DETERMINISTIC.csv`:

#figure(
  caption: [distributions determined from sample file],
  table(
    columns: (auto, auto, auto),
    align: left,
    [*type*],[*distribution*],[*parameters*],
    [inter-arrival time (s)],[exponential],[$lambda = 1.369$],
    [call duration (s)],[2-parameter exponential],[$lambda = 99.831$, $beta = 10.004$],
    [vehicle velocity (km/h)],[normal],[$overline(x) = 120.072$, $sigma = 9.018$],
    [base station (idx)],[discrete uniform],[$a = 1$, $b = 19$],
  )
)

The following other distributions are given in the assignment:
// make a table
#figure(
  caption: [distributions provided in the assignment],
  table(
    columns: (auto, auto, auto),
    align: left,
    [*type*],[*distribution*],[*parameters*],
    [relative vehicle position (m)],[uniform],[$a = 0.0$, $b = 2000.0$],
    [vehicle direction (east, west)],[discrete uniform, binary],[$a = 1$, $b = 2$],
  )
)

== Verification
Distribution parameters are verified by comparing the original data to $1,000,000$ call initiation samples from the simulator (see @appendix).

In all cases, the parameters are within $plus.minus 0.1%$ of the original data.

#figure(
  caption: [measured and generated base station distributions],
  table(
    columns: (auto, auto, auto, auto),
    align: left,
    [*type*],[*measured*],[*generated*],[*difference*],
    [inter-arrival time (s)],[$lambda = 1.369$],[$lambda = 1.370$],[$lambda "+0.07%"$],
    [call duration (s)],[$lambda = 99.831$, $beta = 10.004$],[$lambda = 99.733$, $beta = 10.0044$],[$lambda "+0.09%"$, $beta "+0.004%"$],
    [vehicle velocity (km/h)],[$overline(x) = 120.072$, $sigma = 9.018$],[$overline(x) = 120.066$, $sigma = 9.0179$],[$overline(x) "-0.005%"$, $sigma "-0.0002%"$],
    [base station (idx)],[$a = 1$, $b = 19$],[$a = 1$, $b = 19$],[none],
  )

) <base_station_dist>

= Simulation
The simulation is run for 100,000 iterations in antithetic pairs, and is terminated after 10,000 call initiations are complete.

The first run contains no handover reservations,
while the second run contains 1 handover reservation per base station.

== Warm-up period
To determine the warm-up period, the average utilisation for a station channel is determined experimentally. The warm-up period is any point in time before the average utilisation of a channel stabilizes.

The warm-up period is over once the average utilisation of a channel enters the region $overline(x) plus.minus 1$ channel, where $overline(x)$ is the average utilisation of a channel from initial test simulations.

With reference to @avg_util, the warm-up period is determined to be up to $t = 1000s$
By dividing this number with the average inter-arrival time ($1.36s$), the number of call initiation events to discard is approximately $740$.

#figure(
  caption: [channel availability plot and average utilisation over time],
  image("media/channel_availability_r0.svg")
) <avg_util>

= Results and observations
#linebreak()
// create a summary table
#figure(
  caption: [summary of simulation results],
  table(
    columns: (auto, auto, auto),
    align: left,
    [*handover reservation*], [*blocked calls (%)*],[*dropped calls (%)*],
    [0],[$overline(x) = 0.314$, $sigma = 0.0479$],[$overline(x) = 0.0653$, $sigma = 0.0798$],
    [1],[$overline(x) = 1.022$, $sigma = 0.0930$],[$overline(x) = 0.418$, $sigma = 0.0607$],
  )
)

// math here
At 100,000 samples, the critical t-value for a 99% confidence interval that the results using different channel allocations is significant is $2.808$ (calculated with z = 10,000).

#figure(
  caption: [calculated t-values between results of 0/1 channel allocation schemes],
  table(
    columns: (auto, auto),
    align: left,
    [*dropped calls*], [*blocked calls*],
    [$741.7$],[$2141.0$],
  )
)

From the collected results, it can be noted that the use of 1 channel for handover reservations results in an approximate increase in the percentage of blocked calls from $0.3%$ to $1%$.
The percentage of dropped calls decreases from approximately $0.7%$ to $0.4%$.

The trade off from using 1 reserved handover channel is greater than using no reserved channels.
Approximately $1.4%$ of all calls will encounter either a block or drop event, whereas only $1.0%$ of calls will encounter such events with no reserved channels.

#figure(
  caption: [Percentage distributions of blocked and dropped calls],
  stack(
    dir: ltr,
    image("media/reserve-0.svg", width: 60%),
    image("media/reserve-1.svg", width: 60%)
  )
)

#pagebreak()
= Appendix <appendix>

#let w = 55%

#figure(
  caption: [measured and generated inter-arrival time distributions],
  stack(
    dir: ltr,
    image("media/inter-arrival-time.svg", width: w),
    image("media/simulated-inter-arrival-time.svg", width: w)
  )
) <inter_arrival_time_dist>

#figure(
  caption: [measured and generated call duration distributions],
  stack(
    dir: ltr,
    image("media/call-duration.svg", width: w),
    image("media/simulated-call-duration.svg", width: w)
  )
) <call_duration_dist>

#figure(
  caption: [measured and generated vehicle velocity distributions],
  stack(
    dir: ltr,
    image("media/vehicle-velocity.svg", width: w),
    image("media/simulated-vehicle-velocity.svg", width: w)
  )
) <vehicle_velocity_dist>

#figure(
  caption: [measured and generated base station distributions],
  stack(
    dir: ltr,
    image("media/base-station.svg", width: w),
    image("media/simulated-base-station.svg", width: w)
  )
) <base_station_dist>

#figure(
  caption: [distribution of performance measures, 0 channel reserve],
  stack(
    dir: ltr,
    image("media/reserve-0_blocked.svg", width: w),
    image("media/reserve-0_dropped.svg", width: w)
  )
)

#figure(
  caption: [distribution of performance measures, 1 channel reserve],
  stack(
    dir: ltr,
    image("media/reserve-1_blocked.svg", width: w),
    image("media/reserve-1_dropped.svg", width: w)
  )
)

#linebreak()
#bibliography("assignment_1b.bib")