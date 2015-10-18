use error::{Result, Error, JsonError};
use serde::de;
use serde_json;
use std::fmt;
use time;
use std::collections::HashMap;
use crossroad::*;
use traffic_protocol::*;
use std::sync::mpsc::{channel, Sender, Receiver};


const YELLOW_TIME: i32 = 4;

#[derive(Debug, PartialEq, Clone)]
pub enum TrafficLightState {
    Init,
    MinimalGreen    { start: i32 },
    Green           { start: i32 },
    Yellow          { start: i32 },
    Red,
}

#[derive(Debug, Clone)]
pub struct ControlWithState<'a> {
	pub inner: &'a Control<'a>,
    pub state: TrafficLightState,
    pub force_red: bool,
}

impl <'a>ControlWithState<'a>  {
    pub fn new(inner: &'a Control) -> ControlWithState<'a> {
        ControlWithState { inner:inner, state: TrafficLightState::Init, force_red: false }
    }

    pub fn run_loop(&mut self, time: i32, out_tx: &Sender<String>, sensor_states: &SensorStates, unlimited_green: bool) -> TrafficLightState {

        let new_state = match self.state {

            TrafficLightState::Init => {
                Some(TrafficLightState::MinimalGreen { start: time })
            },

            TrafficLightState::MinimalGreen { start } => {
                println!("****  MinimalGreen since {:?}, {:?} -> {:?}", start,  self.inner, self.state );

                if time >= start + self.inner.traffic_type().min_green() {
                    Some(TrafficLightState::Green{ start: time })
                }
                else {
                    None
                }
            },

            TrafficLightState::Green { start } => {
                println!("****  Green since {:?}, {:?} -> {:?}", start, self.inner, self.state);

                if self.force_red {
                    self.inner.send_unsafe(out_tx, JsonState::Geel);
                    self.force_red = false;
                    Some(TrafficLightState::Yellow{ start: time })
                }
                else if unlimited_green {
                    None
                }
                else {
                    // if: sensor is activated -> extend green time
                    // else if: check if we can move to yellow
                    if sensor_states.has_active(self.inner) {
                        Some(TrafficLightState::Green{ start: time }) // reset timer
                    }
                    else if time >= start + self.inner.traffic_type().green_extra() {
                        self.inner.send_unsafe(out_tx, JsonState::Geel);
                        Some(TrafficLightState::Yellow{ start: time })
                    }
                    else {
                        None
                    }
                }
            },

            TrafficLightState::Yellow { start } => {
                println!("****  Yellow {:?} -> {:?}, {:?} -> {:?}", start, start + YELLOW_TIME, self.inner, self.state);

                if time >= start + YELLOW_TIME {
                    self.inner.send_unsafe(out_tx, JsonState::Rood);
                    Some(TrafficLightState::Red)
                }
                else {
                    None
                }
            },

            TrafficLightState::Red => {
                println!("****  RED");
                None
            }
        };

        if let Some(v) = new_state { self.state = v };
        self.state.clone()
    }
}

// -------------------------------------------------------------------------------
// Control
// -------------------------------------------------------------------------------

#[derive(Debug)]
pub enum Control<'a> {
    Group(&'a TrafficGroup<'a>),
    Single(&'a TrafficLight)
}

impl <'a>From<&'a TrafficGroup<'a>> for Control<'a> {
    fn from(t: &'a TrafficGroup<'a>) -> Control<'a> {
        Control::Group(t)
    }
}
impl <'a>From<&'a TrafficLight> for Control<'a> {
    fn from(t: &'a TrafficLight) -> Control<'a> {
        Control::Single(t)
    }
}

impl <'a>Control<'a> {

    pub fn conflicting_with(&'a self, conflicting_with: Vec<&'a Control>) -> ConflictEntry where Self: Sized {
        ConflictEntry {
            control: self,
            conflicting_with: conflicting_with,
        }
    }

    pub fn is(&self, other: &Control) -> bool {
        let address_self = self as *const Control;
        let address_other = other as *const Control;
        address_self == address_other
    }

    pub fn contains_one_of(&self, ids: &Vec<usize>) -> bool {
        match self {
            &Control::Group(ref group) => group.contains_ids(ids),
            &Control::Single(ref tl) => tl.contains_ids(ids),
        }
    }

    pub fn contains(&self, id: usize) -> bool {
        match self {
            &Control::Group(ref group) => group.contains_id(id),
            &Control::Single(ref tl) => tl.id == id,
        }
    }

    pub fn traffic_type(&self) -> &Type {
        match self {
            &Control::Group(ref group) => &group.traffic_type,
            &Control::Single(ref tl) => &tl.traffic_type,
        }
    }

    pub fn get_ids(&self) -> Vec<usize> {
        match self {
            &Control::Group(ref group) => group.get_ids(),
            &Control::Single(ref tl) => vec![tl.id],
        }
    }

    pub fn direction(&self) -> Direction {
        match self {
            &Control::Group(ref group) => group.direction,
            &Control::Single(ref tl) => tl.direction,
        }
    }

    pub fn json_objs(&self, state: JsonState) -> Vec<StoplichtJson> {
        match self {
            &Control::Group(ref group) => group.json_objs(&state),
            &Control::Single(ref tl) => vec![tl.json_obj(&state)],
        }
    }

    pub fn send_unsafe(&self, out_tx: &Sender<String>, state: JsonState) {
        let json_str = ClientJson::from(self.json_objs(state)).serialize();
        out_tx.send(json_str.unwrap()).unwrap()
    }
}

// -------------------------------------------------------------------------------
// Crossing
// -------------------------------------------------------------------------------

pub struct Crossing<'a> {
    pub bicycle_and_pedestrain: Control<'a>,
    pub inner_pedestrian: Control<'a>,
    pub side: Direction,
}

impl <'a>Crossing<'a> {
    pub fn from(bicycle_and_pedestrain: Control<'a>, inner_pedestrian: Control<'a>, side: Direction) -> Crossing<'a> {
        Crossing {
            bicycle_and_pedestrain: bicycle_and_pedestrain,
            inner_pedestrian: inner_pedestrian,
            side: side,
        }
    }
}

// -------------------------------------------------------------------------------
// ControlSensor
// -------------------------------------------------------------------------------

#[derive(Clone)]
pub struct ControlSensor<'a, 'b> {
    pub inner: &'a Control<'a>,
    pub sensor: &'b Sensor,
    pub conflicting_ids: Vec<usize>
}

impl <'a, 'b>ControlSensor<'a, 'b>  {
    pub fn new(inner: &'a Control, sensor: &'b Sensor, conflicting_ids: Vec<usize>) -> ControlSensor<'a, 'b> {
        ControlSensor { inner:inner, sensor:sensor, conflicting_ids:conflicting_ids }
    }
    pub fn time_waiting(&self, until: time::Tm) -> time::Duration {
        until - self.sensor.last_update
    }
    pub fn filter_conflicting<'c>(&'c self, choices: &'c Vec<ControlSensor<'a, 'b>>) -> Vec<&'c ControlSensor<'a, 'b>> {
        choices.iter()
               .filter(|&choice| !self.conflicting_ids.iter().any(|&id| choice.inner.contains(id)))
               .collect()
    }
}

impl <'a, 'b>fmt::Debug for ControlSensor<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ControlSensor({:?}, sensor: {}, conflicts: {:?})",
            self.inner, self.sensor.last_update.ctime(), self.conflicting_ids)
    }
}


// -------------------------------------------------------------------------------
// TrafficGroup
// -------------------------------------------------------------------------------

pub struct TrafficGroup<'a> {
    pub traffic_lights: Vec<&'a TrafficLight>,
	pub direction: Direction,
    pub traffic_type: Type,
}

impl <'a>TrafficGroup<'a> {
    pub fn with(traffic_lights: Vec<&'a TrafficLight>, traffic_type: Type) -> TrafficGroup {
        TrafficGroup { traffic_lights:traffic_lights, direction:Direction::North, traffic_type:traffic_type }
    }
    pub fn from(traffic_lights: Vec<&'a TrafficLight>, direction: Direction, traffic_type: Type) -> TrafficGroup<'a> {
        TrafficGroup { traffic_lights:traffic_lights, direction:direction, traffic_type:traffic_type }
    }
    pub fn get_ids(&self) -> Vec<usize> {
        self.traffic_lights.iter().map(|tl| tl.id).collect()
    }
    pub fn contains_id(&self, id: usize) -> bool {
        self.traffic_lights.iter().any(|tl| tl.id == id)
    }
    pub fn contains_ids(&self, ids: &Vec<usize>) -> bool {
        self.traffic_lights.iter().any(|tl| ids.iter().any(|&id| id == tl.id))
    }
    pub fn json_objs(&self, state: &JsonState) -> Vec<StoplichtJson> {
        self.traffic_lights.iter().map(|tl| tl.json_obj(state)).collect()
    }
}

impl <'a>fmt::Debug for TrafficGroup<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TrafficGroup({:?},{:?})", self.get_ids(), self.direction)
    }
}


// -------------------------------------------------------------------------------
// TrafficLight
// -------------------------------------------------------------------------------

pub struct TrafficLight {
    pub id: usize,
	pub direction: Direction,
    pub traffic_type: Type,
}

impl TrafficLight {
    pub fn new(id: usize, direction: Direction, traffic_type: Type) -> TrafficLight {
        TrafficLight { id:id, direction:direction, traffic_type:traffic_type}
    }
    pub fn contains_ids(&self, ids: &Vec<usize>) -> bool {
        ids.iter().any(|&id| id == self.id)
    }
    pub fn json_obj(&self, state: &JsonState) -> StoplichtJson {
        StoplichtJson { id: self.id, status: state.id() }
    }
}

impl fmt::Debug for TrafficLight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TrafficLight({:?},{:?})", self.id, self.direction)
    }
}


// -------------------------------------------------------------------------------
// TrafficType
// -------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Type {
    Primary,
    Vehicle,
    Rest
}

impl Type {
    pub fn min_green(&self) -> i32 {
        match self {
            &Type::Primary => 7,
            &Type::Vehicle => 7,
            &Type::Rest => 20,
        }
    }
    pub fn green_extra(&self) -> i32 {
        match self {
            &Type::Primary => 999,//std::i32::MAX,
            &Type::Vehicle => 7,
            &Type::Rest => 10,
        }
    }
}

// -------------------------------------------------------------------------------
// Conflicts
// -------------------------------------------------------------------------------

#[derive(Debug)]
pub struct XorConflictsGroup<'a> {
    pub name: String,
    conflicts: Vec<ConflictEntry<'a>>,
}

impl<'a> XorConflictsGroup<'a> {
    pub fn new(name: String, conflicts: Vec<ConflictEntry<'a>>) -> XorConflictsGroup {
        XorConflictsGroup { name: name, conflicts:conflicts }
    }

    pub fn get_conflicts_for(&'a self, control: &Control<'a>) ->  Option<Vec<usize>> {
        self.conflicts.iter()
            .find(|conflict| conflict.is_for(control))
            .map( |conflict| self.get_conflicting_ids(conflict))
    }

    pub fn get_conflicts_for_id(&'a self, id: usize) ->  Option<Vec<usize>> {
        self.conflicts.iter()
            .find(|conflict| conflict.is_for_id(id))
            .map( |conflict| self.get_conflicting_ids(conflict))
    }

    fn get_conflicting_ids(&self, conflict: &ConflictEntry) -> Vec<usize> {
        let mut top_node_ids = self.top_node_ids();
        top_node_ids.extend(conflict.get_conflicting_ids());
        top_node_ids
    }

    fn top_node_ids(&self) -> Vec<usize> {
        self.conflicts.iter().flat_map(|conflict| conflict.control.get_ids()).collect()
    }
}


#[derive(Debug)]
pub struct ConflictEntry<'a> {
    pub control: &'a Control<'a>,
    pub conflicting_with: Vec<&'a Control<'a>>,
}

impl<'a> ConflictEntry<'a> {
    pub fn is_for(&self, other_control: &'a Control<'a>) -> bool {
        self.control.is(other_control)
    }

    pub fn is_for_id(&self, id: usize) -> bool {
        self.control.contains(id)
    }

    pub fn get_conflicting_ids(&self) -> Vec<usize> {
        self.conflicting_with.iter().map(|x|*x).flat_map(Control::get_ids).collect()
    }
}
