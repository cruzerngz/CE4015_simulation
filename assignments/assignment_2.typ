// cover page
#align(center, text(size: 1.5em, weight: "bold")[

  #image("media/ntu_logo.svg", width: 75%)

  CE4015: Simulation and Modelling
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  2023/2024 Semester 2 Assignment 2:

  Design of a process oriented simulator
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

#set text(size: 11pt, weight: "regular")
#set heading(numbering: "1.1")

= Overview
Design a process-oriented simulator for simulating the utilisation of a cell network over a section of highway.

== Provided Information
1. The highway is a 40km long straight, with 20 stations spaced 2km apart
2. Base stations have a coverage diameter of 2km
3. Base stations do not overlap in their coverage
4. Vehicles can traverse the highway in both directions

== Assumptions
1. Users can initiate a call from any point within the highway, with probability distribution $X$
2. Users enter the highway from either direction with equal probability
3. The position of a vehicle within the coverage of the base station is uniformly distributed
4. The velocity of each vehicle follows a probability distribution $Y$, and stays constant until the vehicle leaves the highway
5. The call duration follows a probability distribution $Z$

== Objectives
1. Design and outline the flow of the process-oriented simulator
2. Describe how the following statistics are collected after the simulation:
  - The number of calls initiated
  - The number of calls dropped
  - The number of calls blocked


= Shared variables

== Counters
1. $N_"calls"$: Total number of calls initiated
2. $C_"dropped"$: Total number of calls dropped
3. $C_"blocked"$: Total number of calls blocked
4. $A_"initiated"$: The number of calls initiated over time
5. $A_"dropped"$: The number of calls dropped over time
6. $A_"blocked"$: The number of calls blocked over time

// can't think of a better name
== Resources
1.

= Process

// init counters and globals
== Initialization


== User process flow


== Base station process flow

== Termination




