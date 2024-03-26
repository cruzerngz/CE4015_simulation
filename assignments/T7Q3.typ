// we makin ppt slides in typst baby

#set page(paper: "presentation-4-3")
#set text(size: 1.75em, font: "Open Sans")
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

= T7Q3
// #linebreak()
// math works
// this is some math -> $1 + 2 = 3$

#text(size: 0.85em)[Ships arrive at a harbor with interarrival times that are IID exponential random
variables with a mean of 1.25 days. The harbor has a dock with only one berth and
one crane for unloading the ships; ships arriving when the berth is occupied join a
FIFO queue. The time for the crane to unload a ship is distributed uniformly
between 0.5 and 1.5 days. The manager of the harbor is interested in the expected
utilization of the crane. While doing simulations, suppose that Stream 1 of the
Random Number Generator is used to generate interarrival times and Stream 2 for
unloading times.

1. Consider using AV for the model. Which input random variates should be generated antithetically, and how could proper synchronization be maintained?

2. Suppose that thought is being given to replace the existing crane with a faster one. The faster crane’s unloading time for a ship would be distributed uniformly between 0.2 and 1.0 days; everything else remains the same. Discuss proper application and implementation of CRN to compare the original system with the proposed new one]
