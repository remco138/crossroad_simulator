use traffic_protocol::*;
use traffic_controls::*;
use error::{Result, Error, JsonError};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use time;
use cartesian;


const HIAAT_TIJD: i32 = 5;
const GARANTIE_ROOD_TIJD: i32 = 2;
trait Geelfase { fn geelfase_lengte(&self) -> i32; }
enum Groenfase { Gemotoriseerd, BussenTrams, Fietsers, Voetgangers, }
impl Groenfase {
    fn time(&self) -> i32 {
        match *self {
            Groenfase::Gemotoriseerd => 7, // 6 - 8 seconden
            Groenfase::BussenTrams   => 5, // 4 - 6 seconden
            Groenfase::Fietsers      => 7, // 5 - 8 seconden
            Groenfase::Voetgangers   => 5, // 4 - 6 seconden
        }
    }
}

// -------------------------------------------------------------------------------
// Direction
// -------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North, East, South, West
}

impl Direction {
    pub fn all() -> Vec<Direction> {
        vec![Direction::North, Direction::East, Direction::South,  Direction::West]
    }
    pub fn retain<'a, 'b, 'c>(&self, src: &Vec<&'c ControlSensor<'a, 'b>>) -> Vec<&'c ControlSensor<'a, 'b>> {
        let mut vec = src.clone();
        vec.retain(|c| c.inner.direction() == *self);
        vec
    }
}

pub fn directions() -> Vec<Direction> {
    vec![Direction::North, Direction::East, Direction::South, Direction::West]
}


// -------------------------------------------------------------------------------
// SignalGroup
// -------------------------------------------------------------------------------

#[derive(Debug)]
pub enum SignalGroupState {
    Start,
    MinimalGreen    { end_time: i32 },
    Green           { end_time: i32 },
    Hiaat           { end_time: i32 },
    Done,
}

#[derive(Debug)]
pub struct SignalGroup<'a>  {
    pub traffic_controls: Vec<&'a Control<'a>>,
    pub state: SignalGroupState,
}

impl<'a> SignalGroup<'a> {

    pub fn new(traffic_controls: Vec<&'a Control>) -> SignalGroup<'a> {
        SignalGroup { traffic_controls:traffic_controls, state: SignalGroupState::Start }
    }

    pub fn empty() -> SignalGroup<'a> {
        SignalGroup { traffic_controls: vec![], state: SignalGroupState::Start }
    }

    pub fn run_loop(&self, time: i32, out_tx: &Sender<String>) -> Option<SignalGroupState> {

        match self.state {

            SignalGroupState::Start => {
                println!("=> Start");

                for control in self.traffic_controls.iter() {
                    println!("---Send {:?} green state", control);

                    //TODO: SEND out_tx
                }

                Some(SignalGroupState::MinimalGreen { end_time: time + 5 })
            },

            SignalGroupState::MinimalGreen { end_time: end } => {
                println!("=> Green time {:?} > end: {:?} = {:?}", time, end, time >= end);

                if time >= end { Some(SignalGroupState::Green{ end_time: time + 7 }) }
                else { None }
            },

            SignalGroupState::Green { end_time: end } => {
                println!("=> Bezig time {:?} > end: {:?} = {:?}", time, end, time >= end);

                if time >= end { Some(SignalGroupState::Hiaat { end_time: time + 7 }) }
                else { None }
            },

            SignalGroupState::Hiaat { end_time: end } => {
                println!("=> Group Done");

                for control in self.traffic_controls.iter() {
                    println!("---Send {:?} red state", control);
                }

                Some(SignalGroupState::Done)
            },

            SignalGroupState::Done => {
                None
            }
        }
    }
}


// -------------------------------------------------------------------------------
// Crossroad
// -------------------------------------------------------------------------------

pub enum CrossroadState<'a> {
    PrimaryTraffic,
    CreateSignalGroup,
    SignalGroup(SignalGroup<'a>),
}

pub struct Crossroad<'a> {
    traffic_controls: Vec<&'a Control<'a>>,
    pub primary_traffic: Vec<&'a Control<'a>>,
    pub secondary_traffic: Vec<&'a Control<'a>>,
    pub directions: HashMap<Direction, XorConflictsGroup<'a>>
}

impl<'a> Crossroad<'a> {

    pub fn new(traffic_lights: &'a Vec<Control<'a>>,
               primary_road_east_2_3: &'a Control<'a>,
               primary_road_west_9_10 : &'a Control<'a>,
               crossing_east:  &'a Crossing<'a>,
               crossing_south: &'a Crossing<'a>,
               crossing_west:  &'a Crossing<'a>,)
            -> Crossroad<'a> {

        let mut directions = HashMap::new();

        directions.insert(Direction::North,
            XorConflictsGroup::from("noord".to_string(), vec![
                traffic_lights[11].conflicting_with(vec![]),
                traffic_lights[6].conflicting_with(vec![
                    &traffic_lights[8], primary_road_west_9_10,  // noord A
                    &traffic_lights[12],                      // noord B
                    primary_road_east_2_3,
                ]),
                traffic_lights[1].conflicting_with(vec![
                    &traffic_lights[8], primary_road_west_9_10,  // noord A
                    &traffic_lights[12],                         // noord B
                    &traffic_lights[13],
                    &traffic_lights[5],
                ]),
            ])
        );

        directions.insert(Direction::East,
            XorConflictsGroup::from("oost".to_string(), vec![
                traffic_lights[7].conflicting_with(vec![]),
                primary_road_east_2_3.conflicting_with(vec![
                    &traffic_lights[5], &traffic_lights[6],   // oost A
                    &traffic_lights[8],                       // oost B
                    &traffic_lights[13]
                ]),
                traffic_lights[12].conflicting_with(vec![
                    &traffic_lights[5], &traffic_lights[6],   // oost A
                    &traffic_lights[8],                       // oost B
                    primary_road_west_9_10,
                    &traffic_lights[1]
                ]),
                crossing_east.bicycle_and_pedestrain.conflicting_with(vec![
                    &traffic_lights[8],
                    primary_road_west_9_10,
                    &traffic_lights[11],
                ]),
            ])
        );

        directions.insert(Direction::South,
            XorConflictsGroup::from("zuid".to_string(), vec![
                traffic_lights[4].conflicting_with(vec![]),
                traffic_lights[13].conflicting_with(vec![
                    &traffic_lights[1], primary_road_east_2_3,  // zuid A
                    &traffic_lights[5],                         // zuid B
                    &traffic_lights[13]
                ]),
                traffic_lights[8].conflicting_with(vec![
                    &traffic_lights[1], primary_road_east_2_3,  // zuid A
                    &traffic_lights[5],                         // zuid B
                    &traffic_lights[6],
                    &traffic_lights[12]
                ]),
                crossing_south.bicycle_and_pedestrain.conflicting_with(vec![
                    &traffic_lights[5],
                    &traffic_lights[6],
                    &traffic_lights[7],
                ]),
            ]),
        );

        directions.insert(Direction::West,
            XorConflictsGroup::from("west".to_string(), vec![
                traffic_lights[14].conflicting_with(vec![]),
                primary_road_west_9_10.conflicting_with(vec![
                    &traffic_lights[12], &traffic_lights[13],   // west A
                    &traffic_lights[1],                         // west B
                    &traffic_lights[6]
                ]),
                traffic_lights[5].conflicting_with(vec![
                    &traffic_lights[12], &traffic_lights[13],   // west A
                    &traffic_lights[1],                         // west B
                    primary_road_east_2_3,
                    &traffic_lights[8]
                ]),
                crossing_west.bicycle_and_pedestrain.conflicting_with(vec![
                    &traffic_lights[1],
                    primary_road_east_2_3,
                    &traffic_lights[4],
                ]),
            ])
        );

        Crossroad {
            traffic_controls: vec![
                &traffic_lights[0],
                &traffic_lights[1],
                primary_road_east_2_3,
                primary_road_east_2_3,
                &traffic_lights[4],
                &traffic_lights[5],
                &traffic_lights[6],
                &traffic_lights[7],
                &traffic_lights[8],
                primary_road_west_9_10,
                primary_road_west_9_10,
                &traffic_lights[11],
                &traffic_lights[12],
                &traffic_lights[13],
                &traffic_lights[14],
                &traffic_lights[15],
                &traffic_lights[16],
                &crossing_west.bicycle_and_pedestrain,
                &traffic_lights[18],    // does not exist !
                &crossing_south.bicycle_and_pedestrain,  // 19
                &crossing_south.bicycle_and_pedestrain,  // 20
                &crossing_east.bicycle_and_pedestrain,   // 21
                &crossing_east.bicycle_and_pedestrain,   // 22
                &crossing_west.bicycle_and_pedestrain,   // 23
                &traffic_lights[24],    // inner pedestrian west
                &crossing_west.bicycle_and_pedestrain,
                &traffic_lights[26],    // inner pedestrian west
                &traffic_lights[27],    // inner pedestrian south
                &crossing_south.bicycle_and_pedestrain,
                &crossing_south.bicycle_and_pedestrain,
                &traffic_lights[30],    // inner pedestrian south
                &crossing_east.bicycle_and_pedestrain,
                &crossing_east.bicycle_and_pedestrain,
                &traffic_lights[33],    // inner pedestrian east
                &traffic_lights[34],    // inner pedestrian east
            ],
            primary_traffic: vec![
                primary_road_east_2_3,
                &traffic_lights[4],
                primary_road_west_9_10,
                &traffic_lights[11],
            ],
            secondary_traffic: vec![
                &traffic_lights[0],
                &traffic_lights[1],
                &traffic_lights[5],
                &traffic_lights[6],
                &traffic_lights[7],
                &traffic_lights[8],
                &traffic_lights[12],
                &traffic_lights[13],
                &traffic_lights[14],
                &crossing_east.bicycle_and_pedestrain,
                &crossing_south.bicycle_and_pedestrain,
                &crossing_west.bicycle_and_pedestrain,
            ],
            directions: directions,
        }
    }

    pub fn run_loop(&'a self, time: i32,
                              state: &CrossroadState<'a>,
                              sensor_shared_state: Arc<Mutex<SensorStates>>,
                              out_tx: &Sender<String>)
                           -> Option<CrossroadState<'a>> {

        match *state {

            CrossroadState::PrimaryTraffic => {
                println!("========== STATE: PrimaryTraffic (default state)");

                let ref mut sensor_state = *sensor_shared_state.lock().unwrap();

                if sensor_state.has_any_active(&self.secondary_traffic) {
                    Some(CrossroadState::CreateSignalGroup)
                }
                else if sensor_state.has_any_active(&self.primary_traffic) {
                    //TODO: not needed, these should be on GREEN by default!
                    Some(CrossroadState::CreateSignalGroup)
                }
                else {
                    None
                }
            },

            CrossroadState::CreateSignalGroup => {
                println!("========== STATE: CreateSignalGroup");

                let ref mut sensor_state = *sensor_shared_state.lock().unwrap();

                let group = self.generate_signalgroup(&sensor_state);

                Some(CrossroadState::SignalGroup(group))
            },

            CrossroadState::SignalGroup(ref group) => {
                print!("========== STATE: SignalGroup ");

                match group.run_loop(time, out_tx) {
                    Some(SignalGroupState::Done) => Some(CrossroadState::PrimaryTraffic),
                    Some(v) => {
                        Some(CrossroadState::SignalGroup(SignalGroup {
                            traffic_controls: group.traffic_controls.clone(),
                            state: v,
                        }))
                    },
                    None => None,
                }
            }
        }
    }

    pub fn generate_signalgroup(&'a self, sensor_state: &SensorStates) -> SignalGroup<'a> {
        let start = time::PreciseTime::now();
        //
        //
        let (longest_waiting, other_active_sensors) = sensor_state.active_and_longest_waiting().unwrap();

        let start_control = self.get_sensor_control(longest_waiting).unwrap();
        let active_controls = self.get_sensor_controls(&other_active_sensors);

        print!("Start sensor:\n  {:?}\nActive sensors:\n  ", start_control);
        for c in &active_controls { print!("{:?}\n  ", c) };

        let compatible_controls = self.choose_compatible(&start_control, &active_controls);
        let signal_group = self.fill_signal_group(&start_control, &compatible_controls);

        println!("\nFinal group\n {:?}", signal_group);
        //
        //
        print!("\nCalculation done in: {:?} milliseconds", start.to(time::PreciseTime::now()).num_milliseconds());
        print!("\n");

        signal_group
    }

    fn fill_signal_group<'b>(&'a self, control: &ControlSensor<'a, 'b>,
                                       compatibles: &Option<Vec<ControlSensor<'a, 'b>>>)
                                   ->  SignalGroup<'a> {

        let mut traffic_controls = vec![control.inner];

        if let &Some(ref compatible_controls) = compatibles {
            let inners: Vec<_> = compatible_controls.iter().map(|c| c.inner).collect();
            traffic_controls.push_all(inners.as_slice());
        }

        SignalGroup::new(traffic_controls)
    }

    fn choose_compatible<'b>(&'a self, control: &ControlSensor<'a, 'b>,
                                       choices: &Vec<ControlSensor<'a, 'b>>)
                                    -> Option<Vec<ControlSensor<'a, 'b>>> {

        let until_now = time::now();

        let non_conflicting = control.filter_conflicting(choices);

        let rest_choices: Vec<Vec<&ControlSensor>> = directions().iter()
                    .map(|d| d.retain(&non_conflicting))
                    .filter(|v| v.len() != 0).collect();

        println!("After conflicting filter: {:?}", control.conflicting_ids);
        for v in &non_conflicting { println!("  {:?}", v) };
        println!("\nRetained direction vecs");
        for v in &rest_choices { println!("{:?}", v) };

        let mut path_results = vec![];
        for control_path in cartesian::combine_list(rest_choices) {
            let mut conflicts = control.conflicting_ids.clone();
            let mut path = vec![];
            let mut acc = time::Duration::zero();

            print!("\n\nchecking control path: ");
            for c in &control_path { print!("{:?} ", c.inner.get_ids()) };

            for current_control in &control_path {

                match current_control.inner.contains_one_of(&conflicts) {
                    true => {
                        print!("\n{:?} is conflicting, ignore", current_control.inner.get_ids())
                    },
                    false => {
                        acc = acc + current_control.time_waiting(until_now);
                        conflicts.push_all(current_control.conflicting_ids.as_slice());
                        path.push(current_control.clone());
                        print!("\n{:?} added. +{:?} seconds. New conflicts: {:?} ", current_control.inner.get_ids(), acc.num_seconds(), conflicts)
                    }
                }
            }

            path_results.push((path.clone(), acc));
         };

         println!("\n\nPath Results:");

         for &(ref path, count) in &path_results {
             print!("Option: Combined waiting time = {:?} seconds for the traffic lights path: ", count.num_seconds());
             for c in path { print!("{:?} ", c.inner.get_ids()) };
             print!("\n");
         }

         path_results.iter()
            .max_by(|&&(ref path, count)| count)
            .map(|&(ref path, count)| {
                path.iter().map(|&c| c.clone()).collect()
            })
    }

    pub fn get_sensor_control<'b>(&'a self, sensor: &'b Sensor) -> Option<ControlSensor<'a, 'b>> {
        self.get_traffic_control(sensor.id).map(|control| {
            ControlSensor::new(control, sensor, self.conflicts_for_tmp(control))
        })
    }

    pub fn get_sensor_controls<'b>(&'a self, sensors: &Vec<&'b Sensor>) -> Vec<ControlSensor<'a, 'b>> {
        sensors.iter().filter_map(|sensor| {
           self.get_traffic_control(sensor.id).map(|control| {
               ControlSensor::new(control, sensor, self.conflicts_for_tmp(control))
           })
        }).collect()
    }
    pub fn conflicts_for_tmp(&'a self, control: &Control<'a>) -> Vec<usize> {
        //TODO: MAYBE CONFLICTS INSIDE THE CONTROLS(trafficlights,etc) ITSELF?!
        println!("Getting conflicts for: {:?}", control);
        self.directions
            .get(&control.direction())
            .and_then(|xor| xor.get_conflicts_for(control))
            .unwrap()
    }
    /*
    pub fn conflicts_for<'b>(&'a self, control: &ControlSensor<'a, 'b>) -> Vec<usize> {
        //TODO: MAYBE CONFLICTS INSIDE THE CONTROLS(trafficlights,etc) ITSELF?!
        self.directions
            .get(control.direction())
            .and_then(|xor| xor.get_conflicts_for_id(control.get_id()))
            .unwrap()
    }*/

    pub fn get_traffic_control_unsafe(&'a self, id: usize) -> &'a Control {
        &*self.traffic_controls[id]
    }

    pub fn get_traffic_control(&'a self, id: usize) -> Option<&'a Control> {
        self.traffic_controls.get(id).map(|c|*c)
    }
}

pub fn generate_traffic_lights() -> Vec<TrafficLight> {
    vec![
        // roads
        TrafficLight::with(0, Direction::North),

        TrafficLight::with(1, Direction::North),
        TrafficLight::with(2, Direction::East),
        TrafficLight::with(3, Direction::East),
        TrafficLight::with(4, Direction::South),

        TrafficLight::with(5, Direction::West),
        TrafficLight::with(6, Direction::North),
        TrafficLight::with(7, Direction::East),

        TrafficLight::with(8, Direction::South),
        TrafficLight::with(9, Direction::West),
        TrafficLight::with(10, Direction::West),
        TrafficLight::with(11, Direction::North),

        TrafficLight::with(12, Direction::East),
        TrafficLight::with(13, Direction::South),
        TrafficLight::with(14, Direction::West),

        // bus
        TrafficLight::with(15, Direction::East),
        TrafficLight::with(16, Direction::West),

        // bycicle
        TrafficLight::with(17, Direction::South),
        TrafficLight::with(18, Direction::South),

        TrafficLight::with(19, Direction::East),
        TrafficLight::with(20, Direction::West),

        TrafficLight::with(21, Direction::South),
        TrafficLight::with(22, Direction::North),

        // pedestrian
        TrafficLight::with(23, Direction::South),
        TrafficLight::with(24, Direction::North),
        TrafficLight::with(25, Direction::North),
        TrafficLight::with(26, Direction::South),

        TrafficLight::with(27, Direction::West),
        TrafficLight::with(28, Direction::East),
        TrafficLight::with(29, Direction::West),
        TrafficLight::with(30, Direction::East),

        TrafficLight::with(31, Direction::North),
        TrafficLight::with(32, Direction::South),
        TrafficLight::with(33, Direction::North),
        TrafficLight::with(34, Direction::South),
    ]
}

pub fn to_controls(v: &Vec<TrafficLight>) -> Vec<Control> {
    v.iter().map(|tl| Control::Single(tl)).collect()
}
