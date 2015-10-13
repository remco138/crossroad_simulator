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
    pub fn retain<'a, 'b>(&self, src: &Vec<ControlWithSensor<'a, 'b>>) -> Vec<ControlWithSensor<'a, 'b>> {
        let mut vec = src.clone();
        vec.retain(|c| c.has_direction(*self));
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
    pub traffic_controls: Vec<&'a TrafficControl>,
    pub state: SignalGroupState,
}

impl<'a> SignalGroup<'a> {

    pub fn new(traffic_controls: Vec<&'a TrafficControl>) -> SignalGroup<'a> {
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
    traffic_controls: Vec<&'a TrafficControl>,
    pub primary_traffic: Vec<&'a TrafficControl>,
    pub secondary_traffic: Vec<&'a TrafficControl>,
    pub directions: HashMap<Direction, XorConflictsGroup<'a>>
}

impl<'a> Crossroad<'a> {

    pub fn new(traffic_lights: &'a Vec<TrafficLight>,
               primary_road_east_2_3: &'a ParallelTrafficControl<'a>,
               primary_road_west_9_10 : &'a ParallelTrafficControl<'a>)
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
                // Crossing South
            ])
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
                &traffic_lights[4],
                &traffic_lights[5],
                &traffic_lights[6],
                &traffic_lights[7],
                &traffic_lights[8],
                &traffic_lights[11],
                &traffic_lights[12],
                &traffic_lights[13],
                &traffic_lights[14],
                &traffic_lights[15],
                &traffic_lights[16],
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
        let active_sensors = sensor_state.active_sensors();
        let active_controls = self.get_sensor_controls(&active_sensors);
        let longest_waiting_sensor = *active_sensors.iter().min_by(|b| b.last_update).unwrap();

        let start_control = active_controls.iter().find(|c| c.has_id(longest_waiting_sensor.id)).unwrap().clone();
        let compatible_controls = self.choose_compatible(&start_control, &active_controls);
        let signal_group = self.fill_signal_group(&start_control, &compatible_controls);

        println!("\n\nRECAP\nStarting control = {:?}", start_control);
        print!("\nAll TrafficControls with an 'ON' sensor:\n ");
        for c in &active_controls { print!("{:?} {:?}, ", c.get_id(), c.get_direction()) };
        print!("\n\nControls compatible with start sensor:\n");
        if let Some(g) = compatible_controls {
            for c in &g { print!("{:?} {:?}, ", c.get_id(), c.get_direction()) };
        }
        println!("\n\nFinal group\n {:?}", signal_group);
        //
        //
        print!("\nCalculation done in: {:?}", start.to(time::PreciseTime::now()).num_milliseconds());
        print!("\n");

        signal_group
    }

    fn fill_signal_group<'b>(&'a self, control: &ControlWithSensor<'a, 'b>,
                                       compatibles: &Option<Vec<ControlWithSensor<'a, 'b>>>)
                                   ->  SignalGroup<'a> {

        let mut traffic_controls = vec![control.inner];

        if let &Some(ref compatible_controls) = compatibles {
            let inners: Vec<_> = compatible_controls.iter().map(|c| c.inner).collect();
            traffic_controls.push_all(inners.as_slice());
        }

        SignalGroup::new(traffic_controls)
    }

    fn choose_compatible<'b>(&'a self, control: &ControlWithSensor<'a, 'b>,
                                       choices: &Vec<ControlWithSensor<'a, 'b>>)
                                    -> Option<Vec<ControlWithSensor<'a, 'b>>> {

        let until_now = time::now();
        let start_conflicts = self.conflicts_for(control);
        let start_direction = control.get_direction();

        println!("\nStart control: {:?}\n", control);
        for v in choices { println!("     choice: {:?}", v) };

        let non_conflicting_choices: Vec<ControlWithSensor> = choices.iter()
                    .filter(|c| !c.has_ids(&start_conflicts))
                    .map(|x|x.clone()).collect();

        let rest_choices: Vec<Vec<ControlWithSensor>> = directions().iter()
                    .map(|d| {d.retain(&non_conflicting_choices)})
                    .filter(|v| v.len() != 0).collect();

        println!("\nRetained vecs");
        for v in &rest_choices { println!("{:?}", v) };

        let mut path_results = vec![];
        for control_path in cartesian::combine_list_v(rest_choices) {
            let mut conflicts = start_conflicts.clone();
            let mut path = vec![];
            let mut acc = time::Duration::zero();

            print!("\n\nchecking control path: ");
            for c in &control_path { print!("{:?} ", c.get_id()) };

            for current_control in &control_path {

                if current_control.has_ids(&conflicts) {
                    print!("\n{:?} is conflicting, ignore", current_control.get_id());
                }
                else {
                    acc = acc + current_control.time_waiting(until_now);
                    conflicts.push_all(self.conflicts_for(&current_control).as_slice()); //TODO: Unique conflict ids? maybe checking if
                    path.push(current_control.clone());
                    print!("\n{:?} added, {:?} new conflicts: {:?} ", current_control.get_id(), acc.num_seconds(), conflicts);
                }
            }

            path_results.push((path.clone(), acc));
         };

         println!("\n\nPath Results:");

         for &(ref path, count) in &path_results {
             print!("Option: Combined waiting time = {:?} seconds for the traffic lights path: ", count.num_seconds());
             for c in path { print!("{:?} ", c.get_id()) };
             print!("\n");
         }

         path_results.iter()
            .max_by(|&&(ref path, count)| count)
            .map(|&(ref path, count)| {
                path.iter().map(|c| c.clone()).collect()
            })
    }

    pub fn get_sensor_controls<'b>(&'a self, sensors: &Vec<&'b Sensor>) -> Vec<ControlWithSensor<'a, 'b>> {
        sensors.iter().filter_map(|sensor| {
           self.get_traffic_control(sensor.id).map(|control| {
               ControlWithSensor::new(control, sensor)
           })
        }).collect()
    }

    pub fn conflicts_for<'b>(&'a self, control: &ControlWithSensor<'a, 'b>) -> Vec<usize> {
        //TODO: MAYBE CONFLICTS INSIDE THE CONTROLS(trafficlights,etc) ITSELF?!
        self.directions
            .get(control.get_direction())
            .and_then(|xor| xor.get_conflicts_for_id(control.get_id()))
            .unwrap()
    }

    pub fn get_traffic_control_unsafe(&'a self, id: usize) -> &'a TrafficControl {
        &*self.traffic_controls[id]
    }

    pub fn get_traffic_control(&'a self, id: usize) -> Option<&'a TrafficControl> {
        self.traffic_controls.get(id).map(|c|*c)
    }
}

pub fn generate_traffic_lights() -> Vec<TrafficLight> {
    vec![
        TrafficLight::with_direction(0, Direction::North),
        TrafficLight::with_direction(1, Direction::North),
        TrafficLight::with_direction(2, Direction::East),
        TrafficLight::with_direction(3, Direction::East),
        TrafficLight::with_direction(4, Direction::South),
        TrafficLight::with_direction(5, Direction::West),
        TrafficLight::with_direction(6, Direction::North),
        TrafficLight::with_direction(7, Direction::East),
        TrafficLight::with_direction(8, Direction::South),
        TrafficLight::with_direction(9, Direction::West),
        TrafficLight::with_direction(10, Direction::West),
        TrafficLight::with_direction(11, Direction::North),
        TrafficLight::with_direction(12, Direction::East),
        TrafficLight::with_direction(13, Direction::South),
        TrafficLight::with_direction(14, Direction::West),
        TrafficLight::with_direction(15, Direction::East),
        TrafficLight::with_direction(16, Direction::West),
        TrafficLight::with_direction(18, Direction::South),
    ]
}
