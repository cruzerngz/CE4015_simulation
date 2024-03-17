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

// import algo package
#import "@preview/algo:0.3.3": algo, i, d, comment, code

#set page(numbering: "1")
#set text(size: 11pt, weight: "regular")
#set heading(numbering: "1.1.")

#outline(title: "Table of Contents", indent: true)
#linebreak()
#outline(title: "Figures", target: figure.where(kind: image))
#pagebreak()

= Overview
Design a process-oriented simulator for simulating the utilisation of a cell network over a section of highway.

== Provided Information
1. The highway is a 40km long straight, with 20 stations spaced 2km apart
2. Base stations have a coverage diameter of 2km
3. Base stations do not overlap in their coverage
4. Vehicles can traverse the highway in both directions

== Input modelling <input_modelling>
1. Users can initiate a call from any base station, with probability distribution $X$
2. Users enter the highway from either direction with equal probability (i.e. 50% chance of entering from the left, 50% chance of entering from the right)
3. The position of a vehicle within the coverage of the base station is uniformly distributed
4. The velocity of each vehicle follows a probability distribution $Y$, and stays constant until the vehicle leaves the highway
5. The call duration follows a probability distribution $Z$

== Objectives
1. Design and outline the flow of the process-oriented simulator
2. Describe how the following statistics are collected after the simulation:
  - The number of calls initiated
  - The number of calls dropped
  - The number of calls blocked

= Shared variables <shared_variables>

== Counters <counters>
1. $N_"calls"$: Total number of calls initiated
2. $C_"dropped"$: Total number of calls dropped
3. $C_"blocked"$: Total number of calls blocked
4. $A_"initiated"$: The number of calls initiated with respect to time
5. $A_"dropped"$: The number of calls dropped with respect to time
6. $A_"blocked"$: The number of calls blocked with respect to time
7. $"BSA"_n$: Base station $n$'s availability with respect to time, $1 <= n <= 20$

// can't think of a better name
== Resources
Each base station $n$, is held behind a mutex. $"BS"_n, 1 <= n <= 20$.
There are 10 available channels for each base station. ($"BS"_"1..20" = 10$ when initialized)

#pagebreak()
= Process
Each step in the process is outlined in the following sections.

// init counters and globals
== Initialization
#algo(
  title: "Init",
  parameters: ("TotalCalls",),
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
  #comment[generate calls based on distributions $X, Y, Z$ and uniform distributions in @input_modelling]
  let EnqueuedCalls = #smallcaps[GenerateCalls]\()\
  EnqueuedCalls.sort()\
  \
  for call in EnqueuedCalls {#i\
    #comment[spawn all pending processes]
    #smallcaps[Spawn]\(#smallcaps[UserProcess]\(call))#d\
  }\

 \
  #comment[clock cycle starts]
  while SystemTime < StopTime {#i\

    if $N_"calls"$ == TotalCalls { *break* }\

    SystemTime = SystemTime + 1\
    #comment[wait until all base station locks are no longer held]
    #smallcaps[WaitUntilFree]\($"BS"_{1..20}$)#d\
  }
]

#pagebreak()
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
  #smallcaps[WaitUntil]\(Call.Start)\ \

  *loop* {#i\
    *match* CallState {#i\
      Initiate $->$ #smallcaps[InitiateCall]\(CurrentBaseStation)\
      Handover $->$ #smallcaps[HandoverCall]\(CurrentBaseStation)\
      Terminate $->$ #smallcaps[TerminateCall]\(CurrentBaseStation)\
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
  #smallcaps[UpdateGraph]\($A_"initiated"$, CurrentTime)\ \

  if #smallcaps[Request]\(BaseStation) succeeds {#i #comment[see @base_station]\
    *pass* #d\
  } else {#i\

    $N_"blocked" = N_"blocked" + 1$\
    #smallcaps[UpdateGraph]\($A_"blocked"$, CurrentTime)\
    #smallcaps[UpdateCallState]\(Complete)\
    return#d\
  }\ \

  let BaseStationBoundary = $"DistanceToCellBoundary" / "VehicleVelocity"$\

  if BaseStationBoundary > CallDuration {#i\

    #comment[check if call is going out of bounds]
    if #smallcaps[Exists]\(BaseStation + NextStation) {#i\

     let HandoverTime = CurrentTime + BaseStationBoundary\

      #smallcaps[UpdateCallState]\(Handover)\
      $"CurrentBaseStation" = "BaseStation" + "NextStation"$\
      $"CallDuration" = "CallDuration" - "BaseStationBoundary"$\
      #comment[go to @call_handover]
      #smallcaps[WaitUntil]\(HandoverTime)
      #d\
    } else {#i\
      let TerminationTime = CurrentTime + BaseStationBoundary\

      #smallcaps[UpdateCallState]\(Terminate)\
      // $"CurrentBaseStation" = "BaseStation" + "NextStation"$\
      $"CallDuration" = 0$\
      #comment[go to @call_termination]
      #smallcaps[WaitUntil]\(TerminationTime)
      #d\
    }

  #d\
  } else {#i\

    let TerminationTime = CurrentTime + CallDuration\
    #smallcaps[UpdateCallState]\(Terminate)\
    $"CallDuration" = 0$\
    #comment[go to @call_termination]
    #smallcaps[WaitUntil]\(TerminationTime)

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
  if #smallcaps[Request]\(BaseStation) succeeds {#i\
    pass #d\
  } else {#i\

    $N_"blocked" = N_"blocked" + 1$\
    #smallcaps[UpdateGraph]\($A_"dropped"$, CurrentTime)\
    #smallcaps[UpdateCallState]\(Complete)\
    return#d\
  }\ \

  #comment[similar logic to @call_initiation]
  if BaseStationBoundary > CallDuration {#i\

    #comment[check if call is going out of bounds]
    if #smallcaps[Exists]\(BaseStation + NextStation) {#i\

     let HandoverTime = CurrentTime + BaseStationBoundary\

      #smallcaps[UpdateCallState]\(Handover)\
      $"CurrentBaseStation" = "BaseStation" + "NextStation"$\
      $"CallDuration" = "CallDuration" - "BaseStationBoundary"$\
      #comment[go to @call_handover]
      #smallcaps[WaitUntil]\(HandoverTime)
      #d\
    } else {#i\
      let TerminationTime = CurrentTime + BaseStationBoundary\

      #smallcaps[UpdateCallState]\(Terminate)\
      // $"CurrentBaseStation" = "BaseStation" + "NextStation"$\
      $"CallDuration" = 0$\
      #comment[go to @call_termination]
      #smallcaps[WaitUntil]\(TerminationTime)
      #d\
    }

  #d\
  } else {#i\

    let TerminationTime = CurrentTime + CallDuration\
    #smallcaps[UpdateCallState]\(Terminate)\
    $"CallDuration" = 0$\
    #comment[go to @call_termination]
    #smallcaps[WaitUntil]\(TerminationTime)

  #d\
  }
]

#pagebreak()
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
  #smallcaps[UpdateCallState]\(Complete)
]

== Base station <base_station>

=== Allocation
A base station awaits for allocation requests and handles a request according to the following:
#figure(
  image("media/proc_base-station.svg", width: 70%),
  caption: [A flow chart of how a base station allocates calls]
)

On successful allocation, the base station will decrement the number of available channels by 1 as well as update the graph $"BSA"_n$:

#algo(
  title: "SuccessfulAllo",
  // parameters: ("StationNumber",),
  block-align: left,
  radius: 3pt
)[
  $"BS"_n = "BS"_n - 1$\
  #smallcaps[UpdateGraph]\($"BSA"_n$, CurrentTime, decrement)\
]

=== Deallocation
During channel deallocation, a base station $n$ awaits a request and performs the following.

#algo(
  title: "Deallocate",
  parameters: ("StationNumber",),
  block-align: left,
  radius: 3pt
)[
  $"BS"_n = "BS"_n + 1$\
  #smallcaps[UpdateGraph]\($"BSA"_n$, CurrentTime, increment)\
]


#linebreak()
== Termination
After all calls have been processed ($N_"calls" == "TotalCalls"$), the simulator will terminate and output statistics defined in @shared_variables.

The following statistics are derived:

1. $P_"dropped" = C_"dropped" / N_"calls"$

2. $P_"blocked" = C_"blocked" / N_"calls"$
