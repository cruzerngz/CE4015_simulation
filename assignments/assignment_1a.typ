// cover page
#align(center, text(size: 1.5em, weight: "bold")[

  #image("media/ntu_logo.svg", width: 75%)

  CE4015: Simulation and Modelling
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  2023/2024 Semester 2 Assignment 1 part 1:

  Design of a discrete event simulator
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

//  CE4015 Simulation and Modelling Assignment 1 part 1

= Overview
1. Design a discrete-event simulator for simulating the cell network service over a section of highway.

2. Present pseudocode and diagrams for various parts of the simulator.

= Design
The simulation consists of 3 discrete events:

#figure(
  image("media/event_flow.svg", width: 55%),
  caption: [
    The general flow of events
  ]
)

#linebreak()

An overview of the simulation program is detailed below:

#figure(
  image("media/overview.svg", width: 75%),
  caption: [
    An overview of the flow of the simulation
  ]
)


#pagebreak()

= Initialization

1. Initialize FEL
2. Initialize completed events list (CEL)
3. Initialize station statistics tracker
4. Generate initial InitiateCall events with the following derived distributions (derived in part 2):
  - Cell tower
  - Position within tower's coverage
  - Inter-call interval time
  - Vehicle velocity
  - Vehicle direction
5. Enqueue events to FEL, sorted by arrival time


= Event handling

The following sections describe each event handling routine.

== Initiate Call
1. Calculate duration to next cell tower, and handle each outcome:
  ```rust
  // station blocks call if there are no available channels
  // this event is executed directly after handling this event
  //
  // NOTE!: If simulating 1-chanel reserve, the number of channels for new calls
  // will be decremented by 1 to simulate 1 reserved channel for handovers
  if available_channels <= 0 {
    enqueue_fel(TerminateCall{
      time: start_time,
      station: curr_station,
    })
  } else {
    available_channels -= 1
  }
  ```
2. Handle the cases for calls that are not terminated (@scheduling)
3. Append the completed event to the Completed Events list (`InitiatedCall`)
  ```rust
  push_to_cel(
    NewCall {
      .. // new call fields
    }
  )
  ```

4. Update cell tower statistics

#pagebreak()

== Handover Call
1. Handle each case corresponding to the number of available channels:
```rust
if available_channels <= 0 {
  enqueue_fel(TerminateCall {
    time: time,
    station: curr_station,
  })
} else {
  // new cell tower
  available_channels -= 1
  // previous cell tower
  available_channels += 1
}
```

2. Handle the cases for calls that are not terminated (@scheduling)
3. Append the completed event to the Completed Events list (`HandoverCall`)
4. Update cell tower statistics

== Terminate Call
1. Match the type of call (New, Handover) and free a channel for a base station
2. Append a completed event to the completed events list (`TerminatedCall`)
3. Update cell tower statistics

== Next event scheduling for accepted calls <scheduling>
The following pseudocode describes the logic for handling calls that have been accepted by
the cell tower. The next event scheduled from here depends on:
- The remaining duration of the call
- The duration to the next cell coverage boundary
- If the call is made at the final base station

```rust
if call_duration < duration_to_next_cell_tower {
  enqueue_fel(TerminateCall{
    time: start_time + call_duration,
    station: curr_station,
  })

} else {
  // end of highway
  if next_station does not exist {
    enqueue_fel(TerminateCall{
      time: start_time + duration_to_next_cell_tower,
      station: curr_station
    })
  // next station exists
  } else {
    enqueue_fel(HandoverCall {
      time: start_time + duration_to_next_cell_tower
      station: next_station
      speed: speed,
      duration: duration - duration_to_next_cell_tower,
      direction: direction,
    })
  }
}
  ```


// TODO: put this at the end for reference
// = Data structures
// The following pseudocode describes the key data structures used in the simulation.

// The base station contains counters for each channel and keeps track of what
// call type it is currently servicing:
// ```go
// type BaseStation struct {
//   max_channels uint
//   // initiated by a user
//   num_new uint
//   // initiated by an adjacent tower
//   num_handover uint


// }
// ```

// Call initiation
// ```rust
// pub struct Event {
//   /// Time when vehicle first enters either end of the highway
//   time: f64,

//   /// Speed of vehicle, km/h
//   velocity: f64,

//   /// Time to next base station.
//   ttn: f64,

//   /// Direction of vehicle
//   direction: VehicleDirection,

//   /// Station where the event has occurred
//   station: BaseStation,

//   // See enum below
//   inner: EventType,
// }

// enum EventType {
//   /// A call is initiated by a customer
//   InitiateCall,

//   /// A call is terminated by a customer
//   TerminateCall,

//   /// A customer's call is passed from one base station to another.
//   HandoverCall,
// }

// ```


