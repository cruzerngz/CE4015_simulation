#set text(
  size: 1.5em,
  weight: "bold"
)

#align(center, [

  #figure(
    image("media/ntu_logo.svg", width: 60%),
  )

  CE4015: Simulation and Modelling
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  #linebreak()
  2023/2024 Semester 1 Assignment 1:

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

#set text(size: 11pt)
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
2. Initialize station statistics tracker
3. Generate initial InitiateCall events with the following derived distributions (derived in part 2):
  - Cell tower
  - Position within tower's coverage
  - Inter-call interval time
  - Vehicle velocity
  - Vehicle direction
4. Enqueue events to FEL, sorted by arrival time


= Event handling

The following sections describe each event handling routine.

== Initiate Call

// === HandleNewCall
1. Calculate duration to next cell tower, and handle each outcome:
  - station fully utilized
  ```rust
  // station blocks call if there are no available channels
  // this event is executed directly after handling this event
  if available_channels == 0 {
    enqueue_fel(TerminateCall{
      time: start_time,
      station: curr_station,
    })
  } else {
    available_channels -= 1
  }

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

  1. append the completed event to the Completed Events list. This is the statistics counter for the simulation:
  ```rust
  push_to_cel(
    NewCall {
      .. // new call fields
    }
  )

  1. Update station utilization statistics
  ```


== Handover Call
1. Process the request according to the FCA scheme:
  - No reservation scheme:
    ```rust

    ```

  - 1-handover reserve:
    ```rust

    ```





== Terminate Call
1. Match the type of call (New, Handover) and decrement the respective counter:
  ```rust

  ```


// TODO: put this at the end for reference
= Data structures
The following pseudocode describes the key data structures used in the simulation.

The base station contains counters for each channel and keeps track of what
call type it is currently servicing:
```go
type BaseStation struct {
  max_channels uint
  // initiated by a user
  num_new uint
  // initiated by an adjacent tower
  num_handover uint


}
```

Call initiation
```rust
pub struct Event {
  /// Time when vehicle first enters either end of the highway
  time: f64,

  /// Speed of vehicle, km/h
  velocity: f64,

  /// Time to next base station.
  ttn: f64,

  /// Direction of vehicle
  direction: VehicleDirection,

  /// Station where the event has occurred
  station: BaseStation,

  // See enum below
  inner: EventType,
}

enum EventType {
  /// A call is initiated by a customer
  InitiateCall,

  /// A call is terminated by a customer
  TerminateCall,

  /// A customer's call is passed from one base station to another.
  HandoverCall,
}

```


