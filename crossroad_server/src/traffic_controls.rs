use error::{Result, Error, JsonError};
use serde::de;
use serde_json;
use std::fmt;
use time;
use std::collections::HashMap;
use crossroad::*;
use traffic_protocol::*;


#[derive(Debug, PartialEq)]
pub enum TrafficLightPhase {
    Red,
    Yellow  { start_time: i32 },
    Green   { start_time: i32 },
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
    pub fn is(&self, other: &Control) -> bool {
        let address_self = self as *const Control;
        let address_other = other as *const Control;

        if address_self == address_other {
            println!("found it by comparing\t\t\t{:?}\n\t\t\t{:?}", address_self, address_other);
        }
        address_self == address_other
    }

    pub fn get_ids(&self) -> Vec<usize> {
        match self {
            &Control::Group(ref group) => group.get_ids(),
            &Control::Single(ref tl) => vec![ tl.id ],
        }
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

    pub fn conflicting_with(&'a self, conflicting_with: Vec<&'a Control>) -> ConflictEntry where Self: Sized {
        ConflictEntry {
            control: self,
            conflicting_with: conflicting_with,
        }
    }

    pub fn direction(&self) -> Direction {
        match self {
            &Control::Group(ref group) => group.direction,
            &Control::Single(ref tl) => tl.direction,
        }
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
}

impl <'a>TrafficGroup<'a> {
    pub fn with(traffic_lights: Vec<&'a TrafficLight>) -> TrafficGroup {
        TrafficGroup { traffic_lights:traffic_lights, direction:Direction::North, }
    }
    pub fn from(traffic_lights: Vec<&'a TrafficLight>, direction: Direction) -> TrafficGroup<'a> {
        TrafficGroup { traffic_lights:traffic_lights, direction:direction, }
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
}

impl <'a>fmt::Debug for TrafficGroup<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TrafficGroup(id[{:?}],{:?})", self.get_ids(), self.direction)
    }
}


// -------------------------------------------------------------------------------
// TrafficLight
// -------------------------------------------------------------------------------

#[derive(Debug)]
pub struct TrafficLight {
    pub id: usize,
    pub phase: TrafficLightPhase,
	pub direction: Direction,
}

impl TrafficLight {
    pub fn new(id: usize) -> TrafficLight {
        TrafficLight { id:id, phase: TrafficLightPhase::Red, direction: Direction::North, }
    }
    pub fn with(id: usize, direction: Direction) -> TrafficLight {
        TrafficLight { id:id, phase: TrafficLightPhase::Red, direction:direction, }
    }
    pub fn contains_ids(&self, ids: &Vec<usize>) -> bool {
        ids.iter().any(|&id| id == self.id)
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
    pub fn from(name: String, conflicts: Vec<ConflictEntry<'a>>) -> XorConflictsGroup {
        XorConflictsGroup { name: name, conflicts:conflicts }
    }

    pub fn get_conflicts_for(&'a self, control: &Control<'a>) ->  Option<Vec<usize>> {
        self.conflicts
            .iter()
            .find(|conflict| conflict.is_for(control))
            .map(|conflict| {
                let mut top_node_ids = self.top_node_ids();
                top_node_ids.extend(conflict.get_conflicting_ids());
                top_node_ids
            })
    }

    pub fn get_conflicts_for_id(&'a self, id: usize) ->  Option<Vec<usize>> {
        self.conflicts
            .iter()
            .find(|conflict| conflict.is_for_id(id))
            .map(|conflict| {
                let mut top_node_ids = self.top_node_ids();
                top_node_ids.extend(conflict.get_conflicting_ids());
                top_node_ids
            })
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
