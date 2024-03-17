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

#set page(numbering: "1")
#set text(size: 11pt, weight: "regular")
#set heading(numbering: "1.1")

// import algo package
#import "@preview/algo:0.3.3": algo, i, d, comment, code

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

== Counters <counters>
1. $N_"calls"$: Total number of calls initiated
2. $C_"dropped"$: Total number of calls dropped
3. $C_"blocked"$: Total number of calls blocked
4. $A_"initiated"$: The number of calls initiated with respect to time
5. $A_"dropped"$: The number of calls dropped with respect to time
6. $A_"blocked"$: The number of calls blocked with respect to time

// can't think of a better name
== Resources
1. $"BS"_n$: Base station $n, 1 <= n <= 20$
2. 10 channels per base station ($"BS"_"1..20" = 10$ when initialized)

#pagebreak()
= Process

// init counters and globals
== Initialization
#algo(
  title: "Init",
  block-align: left,
  radius: 3pt
)[
  SystemTime = 0\

  $N_"calls" = 0$\
  $C_"dropped" = 0$\
  $C_"blocked" = 0$\
  $A_"initiated" = []$\
  $A_"dropped" = []$\
  $A_"blocked" = []$ \
  \
  #comment[initialize base station channels]
  for n in 1..20 {#i\
    $"BS"_n = 10$#d\
  }\
  \
  #comment[generate calls based on distributions $X, Y, Z$]
  let EnqueuedCalls = generate_calls()\
  EnqueuedCalls.sort()\
  \
  for call in EnqueuedCalls {#i\
    spawn(UserProcess(call))#d\
  }\

 \
  #comment[clock cycle starts]
  while SystemTime < StopTime {#i\
    SystemTime = SystemTime + 1\
    Wait(1)#d\
  }
]

== User
The user process can be split into 3 parts:
1. Call initiation
2. Call handover (if necessary)
3. Call termination

#algo(
  title: "UserProcess",
  parameters: ("Call",),
  block-align: left,
  radius: 3pt
)[
  #comment[wait until the call starts]
  WaitUntil(Call.Start)\ \

  *loop* {#i\
    *match* CallState {#i\
      Initiate $->$ InitiateCall(CurrentBaseStation)\
      Handover $->$ HandoverCall(CurrentBaseStation)\
      Terminate $->$ TerminateCall(CurrentBaseStation)\
      Complete $->$ *break* #d\
    }#d\
  }
]


#pagebreak()
=== Call initiation <call_initiation>
The process of initiating a call is as follows:

#algo(
  title: "InitiateCall",
  parameters: ("BaseStation",),
  block-align: left,
  radius: 3pt
)[
  $N_"calls" = N_"calls" + 1$ \
  update_graph($A_"initiated"$, CurrentTime)\ \

  if request(BaseStation) succeeds {#i #comment[see @base_station]\
    pass #d\
  } else {#i\

    $N_"blocked" = N_"dropped" + 1$\
    update_graph($A_"dropped"$, CurrentTime)\

    return#d\
  }\ \

  let BaseStationBoundary = $"DistanceToCellBoundary" / "VehicleVelocity"$\

  if BaseStationBoundary > CallDuration and \
    exists(BaseStation + NextStation) {#i\
    let HandoverTime = CurrentTime + BaseStationBoundary\

    update_call_state(Handover)\
    $"CurrentBaseStation" = "BaseStation" + "NextStation"$\
    $"CallDuration" = "CallDuration" - "BaseStationBoundary"$\
    #comment[go to @call_handover]
    WaitUntil(HandoverTime)
  #d\
  } else {#i\

    let TerminationTime = CurrentTime + CallDuration\
    update_call_state(Terminate)\
    $"CallDuration" = 0$\
    #comment[go to @call_termination]
    WaitUntil(TerminationTime)

  #d\
  }
]

#pagebreak()
=== Call handover <call_handover>
The process of handing over a call is as follows:

#algo(
  title: "HandoverCall",
  parameters: ("BaseStation",),
  block-align: left,
  radius: 3pt
)[
  if request(BaseStation) succeeds {#i\
    pass #d\
  } else {#i\

    $N_"blocked" = N_"blocked" + 1$\
    update_graph($A_"blocked"$, CurrentTime)\

    return#d\
  }\ \

  #comment[same logic as @call_initiation]
  let BaseStationBoundary = $"DistanceToCellBoundary" / "VehicleVelocity"$\

  if BaseStationBoundary > CallDuration and \
    exists(BaseStation + NextStation) {#i\
    let HandoverTime = CurrentTime + BaseStationBoundary\

    update_call_state(Handover)\
    $"CurrentBaseStation" = "BaseStation" + "NextStation"$\
    $"CallDuration" = "CallDuration" - "BaseStationBoundary"$\
    #comment[go to @call_handover]
    WaitUntil(HandoverTime)
  #d\
  } else {#i\

    let TerminationTime = CurrentTime + CallDuration\
    update_call_state(Terminate)\
    $"CallDuration" = 0$\
    #comment[go to @call_termination]
    WaitUntil(TerminationTime)

  #d\
  }
]

=== Call termination <call_termination>
The process of terminating a call is as follows:

#algo(
  title: "TerminateCall",
  parameters: ("BaseStation",),
  block-align: left,
  radius: 3pt
)[
  #comment[terminations will always succeed]
  release(BaseStation)\
  update_call_state(Complete)
]

== Base station <base_station>
A base station awaits for allocation requests and handles a request according to the following:
#figure(
  image("media/proc_base-station.svg", width: 70%),
  caption: [A flow chart of how a base station allocates calls]
)

#linebreak()
During channel deallocation, a base station $n$ awaits a request and performs the following:

$"BS"_n = "BS"_n + 1$: Increment the number of available channels for base station $n$

== Termination
After all calls have been processed, the simulator will terminate and output the statistics.

