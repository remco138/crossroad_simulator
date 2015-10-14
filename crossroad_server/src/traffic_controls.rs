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

pub trait TrafficControl: fmt::Debug {
    fn get_id(&self) -> usize;
    fn has_id(&self, id: usize) -> bool;
    fn has_ids(&self, ids: &Vec<usize>) -> bool;
	fn conflicting_with<'a>(&'a self, conflicting_with: Vec<&'a TrafficControl>) -> ConflictEntry where Self: Sized {
		ConflictEntry {
			traffic_control: self,
			conflicting_with: conflicting_with,
		}
	}
    fn has_direction(&self, direction: Direction) -> bool;
    fn get_direction(&self) -> &Direction;
}


#[derive(Clone)]
pub struct ControlWithSensor<'a, 'b> {
    pub inner: &'a TrafficControl,
    pub sensor: &'b Sensor,
}

impl<'a, 'b> ControlWithSensor<'a, 'b>  {
    pub fn new(inner: &'a TrafficControl, sensor: &'b Sensor) -> ControlWithSensor<'a, 'b> {
        ControlWithSensor { inner:inner, sensor:sensor }
    }
    pub fn time_waiting(&self, until: time::Tm) -> time::Duration {
        until - self.sensor.last_update
    }
}

impl <'a, 'b>TrafficControl for ControlWithSensor<'a, 'b> {
    fn get_id(&self) -> usize {
        self.inner.get_id()
    }
    fn has_id(&self, id: usize) -> bool {
        self.inner.has_id(id)
    }
    fn has_ids(&self, ids: &Vec<usize>) -> bool {
        self.inner.has_ids(ids)
    }
    fn has_direction(&self, direction: Direction) -> bool {
        self.inner.has_direction(direction)
    }
    fn get_direction(&self) -> &Direction {
        self.inner.get_direction()
    }
}

impl <'a, 'b>fmt::Debug for ControlWithSensor<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ControlWithSensor({:?}, sensor: {})", self.inner, self.sensor.last_update.ctime())
    }
}


pub struct ParallelTrafficControl<'a> {
    pub controls: Vec<&'a TrafficControl>,
    pub fase: TrafficLightPhase,
	pub direction: Direction,
}

impl<'a> ParallelTrafficControl<'a> {
    pub fn from(controls: Vec<&'a TrafficControl>, direction: Direction) -> ParallelTrafficControl<'a> {
        ParallelTrafficControl {
            controls:controls,
            fase: TrafficLightPhase::Red,
			direction:direction,
        }
    }
}

impl <'a>TrafficControl for ParallelTrafficControl<'a> {
    fn has_id(&self, id: usize) -> bool {
        self.controls.iter().any(|c| c.has_id(id))
    }
    fn get_id(&self) -> usize {
        self.controls[0].get_id()
    }
    fn has_ids(&self, ids: &Vec<usize>) -> bool {
        self.controls.iter().any(|c| ids.iter().any(|id| c.has_id(*id)))
    }
    fn has_direction(&self, direction: Direction) -> bool {
        self.controls.iter().any(|c| c.has_direction(direction))
    }
    fn get_direction(&self) -> &Direction {
        &self.direction
    }
}

impl <'a>fmt::Debug for ParallelTrafficControl<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ids: Vec<usize> = self.controls.iter().map(|c| c.get_id()).collect();
        write!(f, "ParallelTrafficControl(id[{:?}],{:?})", ids, self.direction)
    }
}


#[derive(Debug)]
pub struct TrafficLight {
    pub id: usize,
    pub fase: TrafficLightPhase,
	pub direction: Direction,
}

impl TrafficLight {
    pub fn new(id: usize) -> TrafficLight {
        TrafficLight {id:id, fase: TrafficLightPhase::Red, direction: Direction::North,}
    }
    pub fn with_direction(id: usize, direction: Direction) -> TrafficLight {
        TrafficLight {
			id:id,
            fase: TrafficLightPhase::Red,
            direction:direction,
		}
    }
}

impl TrafficControl for TrafficLight {
    fn get_id(&self) -> usize {
        self.id
    }
    fn has_id(&self, id: usize) -> bool {
        self.id == id
    }
    fn has_ids(&self, ids: &Vec<usize>) -> bool {
        ids.iter().any(|id| *id == self.id)
    }
    fn has_direction(&self, direction: Direction) -> bool {
        self.direction == direction
    }
    fn get_direction(&self) -> &Direction {
        &self.direction
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

    pub fn get_conflicts_for_id(&'a self, id: usize) ->  Option<Vec<usize>> {
        let selected_conflict = self.conflicts.iter().find(|c| c.traffic_control.has_id(id));

        selected_conflict.map(|conflict| {
            let mut top_node_ids:Vec<usize> = self.conflicts.iter().map(|cf| cf.traffic_control.get_id()).collect();
            let extra_confl_ids = conflict.get_conflict_ids();
            top_node_ids.extend(extra_confl_ids);
            top_node_ids
        })
    }

    pub fn get_traffic_controls(&self) -> Vec<&'a TrafficControl> {
        self.conflicts.iter().map(|cf| cf.traffic_control).collect()
    }
}


#[derive(Debug)]
pub struct ConflictEntry<'a> {
    pub traffic_control: &'a TrafficControl,
    pub conflicting_with: Vec<&'a TrafficControl>,
}

impl<'a> ConflictEntry<'a> {
    pub fn get_conflict_ids(&self) -> Vec<usize> {
        self.conflicting_with.iter().map(|vr| vr.get_id()).collect()
    }

    pub fn from(src: &'a ConflictEntry<'a>, extra: Vec<&'a TrafficControl>) -> ConflictEntry<'a> {
        let mut inst = ConflictEntry {
            traffic_control: src.traffic_control,
            conflicting_with: src.conflicting_with.clone(),
        };
        inst.conflicting_with.push_all(extra.as_slice());
        inst
    }
}
