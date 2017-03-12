use std::fmt;
use time;
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
                //println!("****  MinimalGreen since {:?}, {:?} -> {:?}", start,  self.inner, self.state );

                if time >= start + self.inner.traffic_type().min_green() {
                    Some(TrafficLightState::Green{ start: time })
                }
                else {
                    None
                }
            },

            TrafficLightState::Green { start } => {
                //println!("****  Green since {:?}, {:?} -> {:?}", start, self.inner, self.state);

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
                        println!(":::: Extending green timer for: {:?}" ,self.inner.get_ids());
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
                //println!("****  Yellow {:?} -> {:?}, {:?} -> {:?}", start, start + YELLOW_TIME, self.inner, self.state);

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

#[derive(Debug, Eq, Hash)]
pub enum Control<'a> {
    Group(TrafficGroup<'a>),
    Single(&'a TrafficLight)
}

impl <'a>PartialEq for Control<'a> {
    fn eq(&self, other: &Control) -> bool {
        let address_self = self as *const Control;
        let address_other = other as *const Control;
        address_self == address_other
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
        let json_str = out_compat_json_str(self.json_objs(state));
        out_tx.send(json_str).unwrap()
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
// ControlsBuilder
// -------------------------------------------------------------------------------

pub struct ControlsBuilder<'a> {
    pub tlights: &'a TrafficLightsBuilder,
    pub groups: Vec<TrafficGroup<'a>>,
}

impl <'a>ControlsBuilder<'a> {
    pub fn new(tlights: &'a TrafficLightsBuilder) -> ControlsBuilder<'a> {
        ControlsBuilder { tlights: tlights, groups: vec![] }
    }
    pub fn add_group(mut self, ids: Vec<usize>, d: Direction, t: Type) -> Self {
        let traffic_lights = ids.into_iter().map(|id| &self.tlights.traffic_lights[id]).collect();
        self.groups.push(TrafficGroup::from(traffic_lights, d, t));
        self
    }

    pub fn create_controls(self) -> Vec<Control<'a>> {
        let mut tl_controls = self.tlights.as_controls();

        // Process each group
        for group in self.groups.into_iter() {

            // Remove each id in the group from tl_controls
            for id in group.get_ids() {
                if let Some(index) = tl_controls.iter().position(|tlc| tlc.contains(id)) {
                    println!("Removing tl control @ index {:?}, id: {:?}", index, id);
                    tl_controls.remove(index);
                }
            }

            // Add the group as a Control
            tl_controls.push(Control::Group(group));
        }

        tl_controls
    }
}

pub fn index_controls<'a, 'b>(input: &'b Vec<Control<'a>>) -> Vec<&'b Control<'a>> {
    let length = input.iter().flat_map(|c| c.get_ids()).max().unwrap();
    let mut vec = Vec::with_capacity(length);

    for i in 0..length+1 {
        vec.push(input.iter().find(|c| c.contains(i)).unwrap());
        println!("      index {:?} = {:?}", i ,vec[i] );
    }

    vec
}

// -------------------------------------------------------------------------------
// TrafficGroup
// -------------------------------------------------------------------------------

#[derive(PartialEq, Eq, Hash)]
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

#[derive(PartialEq, Eq, Hash)]
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
// TrafficLightsBuilder
// -------------------------------------------------------------------------------

pub struct TrafficLightsBuilder {
    pub traffic_lights: Vec<TrafficLight>,
}

impl <'a>TrafficLightsBuilder {
    pub fn new(count: usize) -> TrafficLightsBuilder {
        TrafficLightsBuilder { traffic_lights: (0..count+1).map(|i| TrafficLight::new(i, Direction::North, Type::Vehicle)).collect() }
    }
    pub fn set_direction(mut self, d: Direction, ids: Vec<usize>) -> TrafficLightsBuilder {
        for id in ids { self.traffic_lights[id].direction = d }
        self
    }
    pub fn set_type(mut self, t: Type, ids: Vec<usize>) -> TrafficLightsBuilder {
        for id in ids { self.traffic_lights[id].traffic_type = t }
        self
    }
    pub fn set_type_range(mut self, t: Type, from: usize, to: usize) -> TrafficLightsBuilder {
        for id in from..to { self.traffic_lights[id].traffic_type = t }
        self
    }

    pub fn as_controls(&'a self) -> Vec<Control<'a>> {
        self.traffic_lights.iter().map(|tl| Control::Single(tl)).collect()
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
    pub fn retain<'a, 'b, 'c>(&self, src: &Vec<&'c ControlSensor<'a, 'b>>) -> Vec<&'c ControlSensor<'a, 'b>> {
        let mut vec = src.clone();
        vec.retain(|c| c.inner.direction() == *self);
        vec
    }
}

pub fn directions() -> Vec<Direction> {
    vec![Direction::North, Direction::East, Direction::South, Direction::West]
}

pub fn split_by_directions<'a, 'b, 'c>(controls: &Vec<&'c ControlSensor<'a, 'b>>) -> Vec<Vec<&'c ControlSensor<'a, 'b>>> {
    directions().iter()
                .map(|d| d.retain(controls))
                .filter(|v| v.len() != 0).collect()
}


// -------------------------------------------------------------------------------
// TrafficType
// -------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Primary,
    Vehicle,
    Rest
}

impl Type {
    pub fn min_green(&self) -> i32 {
        match self {
            &Type::Primary => 5,
            &Type::Vehicle => 5,
            &Type::Rest => 10,
        }
    }
    pub fn green_extra(&self) -> i32 {
        match self {
            &Type::Primary => 999,//std::i32::MAX,
            &Type::Vehicle => 3,
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

    pub fn get_conflicting_ids(&self) -> Vec<usize> {
        self.conflicting_with.iter().map(|x|*x).flat_map(Control::get_ids).collect()
    }
}
