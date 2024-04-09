// we makin ppt slides in typst baby

#set page(paper: "presentation-4-3", margin: 2cm)
#set text(size: 1.75em, font: "Source Sans 3")
#show heading: head => {
  block(
    breakable: false,
    height: 1em,
    below: 1em,
    width: 100%,
    stroke: (
      bottom: (
        paint: black,
        thickness: 1pt,
        dash: "densely-dotted",
      )
    ),
    // outset: 0.2em,
    // fill: luma(200)
  )[#head]
}

// for inline code formatting
#show raw.where(lang: "py"): code => {
  block(
    fill: luma(247),
    radius: 3pt,
    outset: 5pt,
    stroke: black,
    breakable: false,
    width: 100%, // does this look nice?
  )[#text(size: 0.8em)[#code]]
}

= T7Q3
// #linebreak()
// math works
// this is some math -> $1 + 2 = 3$

#text(size: 0.85em)[Ships arrive at a harbor with inter-arrival times that are independently and identically distributed (IID) exponential random variables with a mean of 1.25 days.
The harbor has a dock with only one berth and one crane for unloading the ships;
ships arriving when the berth is occupied join a FIFO queue.
The time for the crane to unload a ship is distributed uniformly between 0.5 and 1.5 days.
The manager of the harbor is interested in the expected utilization of the crane.
While doing simulations, suppose that Stream 1 of the
Random Number Generator is used to generate inter-arrival times and Stream 2 for
unloading times.

1. Consider using AV for the model. Which input random variates should be generated antithetically, and how could proper synchronization be maintained?

2. Suppose that thought is being given to replace the existing crane with a faster one. The faster crane’s unloading time for a ship would be distributed uniformly between 0.2 and 1.0 days; everything else remains the same. Discuss proper application and implementation of commmon random numbers (CRN) to compare the original system with the proposed new one]

#pagebreak()
= Distributions
#align(
  horizon,
  figure(
    stack(
    dir: ltr,
    image("media/expon_cdf.svg", width: 55%),
    image("media/uniform_cdf.svg", width: 55%)
  )
))

#pagebreak()
= Performance measure: crane utilization
```py
# large interarrival times decrease crane utilization
# small interarrival times increase crane utilization
#
# large crane service timees increase crane utilization
# small crane service times decrease crane utilization (more waiting)
ship_arrival_times = [0.2, 0.5, 1.5] # and so on
crane_service_times = [0.5, 1.0, 1.3] # and so on

crane_service_wait = []

for arr_time, service_time in zip(ship_arrival_times, crane_service_times):
    if arr_time > service_time:
        crane_service_wait.append(arr_time)
    else:
        crane_service_wait.append(service_time)

utilization = sum(crane_service_times) / sum(crane_service_wait)

# utilization = 0.93 (93%)
```

#pagebreak()
= Performance measure: crane utilization
#figure(
  image("media/util_rate.svg", width: 80%)

)

#pagebreak()
= Antithetic Variates (AV)
1. Ship interarrival times
  - monotonic
  - large inter-arrival times decrease crane utilization
  - small inter-arrival times increase crane utilization

#linebreak()
2. Crane unloading times
  - monotonic
  - large crane service times increase crane utilization
  - small crane service times decrease crane utilization


= AV synchronization
- Perform simulation runs in pairs
- Stream 1: ship interarrival times
- Stream 2: crane unloading times


#pagebreak()
= Comparing systems
To be fixed between both system simulations:
- Run length (e.g. 100 ships for terminating sim)
- Num of replications (e.g. 1000 paired replications)
- *Ship interarrival time distribution* (commmon random numbers)
- Performance measure (e.g. crane utilizations over num replications)

To be changed:
- Crane service time distribution


#pagebreak()
= Comparing systems
// Calculate the average crane utilization for both systems:

sample mean 1: $theta_1 = 1/n sum_"r=1"^n X_"1r"$ ..and so on

// sample mean 2: $theta_2 = 1/n sum_"r=1"^n X_"2r"$

sample variance 1,2: formula too complicated, take from slides

some confidence interval $alpha = 0.1?$

#linebreak()

#figure(
  // caption: text(size: 0.8em)[a],
  image("media/comp_right.png", width: 52%)
)
#figure(
  image("media/comp_center.png", width: 52%)
)
#figure(
  image("media/comp_left.png", width: 52%)
)
