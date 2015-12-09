use traffic_protocol::*;
use traffic_controls::*;
use signal_group::*;
use error::{Result, Error, JsonError};
use serde_json;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use time;
use cartesian;
use std::ops::Deref;


pub enum CrossroadState<'a> {
    AllRed,
    PrimaryTraffic(SignalGroup<'a>),
    CreatePriorityGroup,
    CreateSignalGroup,
    SignalGroup(SignalGroup<'a>),
}

pub struct Crossroad<'a> {
    pub traffic_controls: Vec<&'a Control<'a>>,
    pub primary_group: SignalGroup<'a>,
    pub primary_traffic: Vec<&'a Control<'a>>,
    pub secondary_traffic: Vec<&'a Control<'a>>,
    pub priority_traffic: Vec<&'a Control<'a>>,
    pub directions: HashMap<Direction, XorConflictsGroup<'a>>
}

impl<'a> Crossroad<'a> {

    pub fn run_loop(&'a self, time: i32,
                              state: &mut CrossroadState<'a>,
                              sensor_shared_state: Arc<Mutex<SensorStates>>,
                              out_tx: &Sender<String>)
                           -> Option<CrossroadState<'a>> {

        let ref mut sensor_states = *sensor_shared_state.lock().unwrap();

        match *state {

            CrossroadState::AllRed => {
                println!("========== STATE: AllRed");

                if sensor_states.has_any_active(&self.priority_traffic) {
                    Some(CrossroadState::CreatePriorityGroup)
                }
                else if sensor_states.has_any_active(&self.secondary_traffic) {
                    Some(CrossroadState::CreateSignalGroup)
                }
                else {
                    Some(CrossroadState::PrimaryTraffic(self.primary_group.clone()))
                }
            },

            CrossroadState::PrimaryTraffic(ref mut group) => {
                print!("========== STATE: PrimaryTraffic ");
                let group_is_green = group.controls_have_state(TrafficLightState::Green{start:0});
                let any_sensor_active = sensor_states.has_any_active(&self.secondary_traffic) || sensor_states.has_any_active(&self.priority_traffic);

                if group_is_green && any_sensor_active {
                    println!(" :)))) Secondary traffic detected. Closing main traffic lanes");
                    Some(CrossroadState::SignalGroup(group.clone_with(SignalGroupState::ForceRed)))
                }
                else {
                    match group.run_loop(time, out_tx, &sensor_states) {
                        Some(SignalGroupState::Done) => Some(CrossroadState::AllRed),
                        Some(v) => Some(CrossroadState::PrimaryTraffic(group.clone_with(v))),
                        None => None,
                    }
                }
            },

            CrossroadState::CreatePriorityGroup => {
                println!("========== STATE: CreatePriorityGroup");
                None
            },

            CrossroadState::CreateSignalGroup => {
                println!("========== STATE: CreateSignalGroup");
                let group = self.generate_signalgroup(&sensor_states);
                Some(CrossroadState::SignalGroup(group))
            },

            CrossroadState::SignalGroup(ref mut group) => {
                print!("========== STATE: SignalGroup ");
                match group.run_loop(time, out_tx, &sensor_states) {
                    Some(SignalGroupState::Done) => Some(CrossroadState::AllRed),
                    Some(v) => Some(CrossroadState::SignalGroup(group.clone_with(v))),
                    None => None,
                }
            }
        }
    }

    pub fn generate_signalgroup(&'a self, sensor_states: &SensorStates) -> SignalGroup<'a> {
        let start = time::PreciseTime::now();
        //
        //
        let (longest_waiting, other_active_sensors) = sensor_states.active_and_longest_waiting().expect("massive boner get_sensor_control");

        let start_control = self.get_sensor_control(longest_waiting).expect("generate_signalgroup get_sensor_control");
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

    fn choose_compatible<'b>(&'a self, control: &ControlSensor<'a, 'b>,
                                       choices: &Vec<ControlSensor<'a, 'b>>)
                                    -> Option<Vec<ControlSensor<'a, 'b>>> {

        let until_now = time::now();
        let non_conflicting = control.filter_conflicting(choices);
        let rest_choices = split_by_directions(&non_conflicting);

        println!("\nAfter conflicting filter: {:?}", control.conflicting_ids);
        for v in &non_conflicting { println!("  {:?}", v) };
        println!("\nRetained direction vecs");
        for v in &rest_choices { println!("{:?}", v) };

        let mut path_results = vec![];
        for control_path in cartesian::all_possibilities(rest_choices) {
            let mut conflicts = control.conflicting_ids.clone();
            let mut path = vec![];
            let mut acc = time::Duration::zero();

            print!("\nchecking control path: ");
            for c in &control_path { print!("{:?} ", c.inner.get_ids()) };

            for current_control in &control_path {
                match current_control.inner.contains_one_of(&conflicts) {
                    true  => { print!("\n{:?} is conflicting, ignore", current_control.inner.get_ids()) },
                    false => {
                        acc = acc + current_control.time_waiting(until_now);
                        conflicts.push_all(current_control.conflicting_ids.as_slice());
                        path.push(current_control.clone());
                        //print!("\n{:?} added. +{:?} seconds. New conflicts: {:?} ", current_control.inner.get_ids(), acc.num_seconds(), conflicts)
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

    fn fill_signal_group<'b>(&'a self, control: &ControlSensor<'a, 'b>,
                                       compatibles: &Option<Vec<ControlSensor<'a, 'b>>>)
                                   ->  SignalGroup<'a> {

        let mut traffic_controls = vec![control.inner];

        if let &Some(ref compatible_controls) = compatibles {
            let inners: Vec<_> = compatible_controls.iter().map(|c| c.inner).collect();
            traffic_controls.push_all(inners.as_slice());
        }

        SignalGroup::new(traffic_controls, false)
    }

    pub fn get_sensor_control<'b>(&'a self, sensor: &'b Sensor) -> Option<ControlSensor<'a, 'b>> {
        self.traffic_controls.get(sensor.id).map(|control| {
            ControlSensor::new(control, sensor, self.conflicts_for(control))
        })
    }

    pub fn get_sensor_controls<'b>(&'a self, sensors: &Vec<&'b Sensor>) -> Vec<ControlSensor<'a, 'b>> {
        sensors.iter().filter_map(|sensor| {
           self.traffic_controls.get(sensor.id).map(|control| {
               ControlSensor::new(control, sensor, self.conflicts_for(control))
           })
        }).collect()
    }

    pub fn conflicts_for(&'a self, control: &Control<'a>) -> Vec<usize> {
        //println!("Getting conflicts for: {:?}", control);
        match self.directions
                  .get(&control.direction())
                  .and_then(|xor| xor.get_conflicts_for(control)) {
            Some(conflicts) => conflicts,
            None => {
                println!("!!!!!!!!!!!!!--------------------> no conflicts for id(s): {:?}", control.get_ids());
                vec![]
            }
        }
    }

    pub fn traffic_controls_unique(&'a self) -> HashSet<&'a Control<'a>> {
        self.traffic_controls.clone().into_iter().collect()
    }

    pub fn send_all(&'a self, out_tx: &Sender<String>, state: JsonState) {
        for control in &self.traffic_controls_unique() {
            control.send_unsafe(out_tx, state)
        }
    }

    pub fn send_all_bulk(&'a self, out_tx: &Sender<String>, state: JsonState) {
        let all_objs = self.traffic_controls_unique().iter().flat_map(|c| c.json_objs(state)).collect();
        let json_str = out_compat_json_str(all_objs);
        out_tx.send(json_str).unwrap();
    }
}
