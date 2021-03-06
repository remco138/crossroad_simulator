use traffic_protocol::*;
use traffic_controls::*;
use std::sync::mpsc::{Sender};
use std::intrinsics;


#[derive(Debug, Clone)]
pub enum SignalGroupState {
    Start,
    Busy { start: i32 },
    ForceRed,
    Done,
}

#[derive(Debug, Clone)]
pub struct SignalGroup<'a>  {
    pub controls: Vec<ControlWithState<'a>>,
    pub state: SignalGroupState,
    pub unlimited_green: bool,
    pub max_green: i32,
    pub is_bus: bool,
}

const MAX_GREEN_TEMP: i32 = 15;

impl<'a> SignalGroup<'a> {

    pub fn new(controls: Vec<&'a Control>, unlimited_green: bool) -> SignalGroup<'a> {
        SignalGroup {
            controls: controls.iter().map(|c| ControlWithState::new(c)).collect(),
            state: SignalGroupState::Start,
            unlimited_green: unlimited_green,
            max_green: MAX_GREEN_TEMP,
            is_bus: false,
        }
    }
    pub fn new_bus(controls: Vec<&'a Control>, unlimited_green: bool) -> SignalGroup<'a> {
        SignalGroup {
            controls: controls.iter().map(|c| ControlWithState::new(c)).collect(),
            state: SignalGroupState::Start,
            unlimited_green: unlimited_green,
            max_green: MAX_GREEN_TEMP,
            is_bus: true,
        }
    }

    pub fn empty() -> SignalGroup<'a> {
        SignalGroup {
            controls: vec![],
            state: SignalGroupState::Start,
            unlimited_green: false,
            max_green: MAX_GREEN_TEMP,
            is_bus: false,
         }
    }

    pub fn clone_with(&self, state: SignalGroupState) -> SignalGroup<'a> {
        SignalGroup {
            controls: self.controls.clone(),
            state: state,
            unlimited_green: self.unlimited_green,
            max_green: self.max_green,
            is_bus: self.is_bus
        }
    }

    pub fn run_loop(&mut self, time: i32, out_tx: &Sender<String>, sensor_states: &SensorStates) -> Option<SignalGroupState> {

        match self.state {

            SignalGroupState::Start => {
                println!("=> Starting ControlGroup");
                let json_str = match self.is_bus {
                    true  => out_compat_json_str(self.construct_bulk_json(JsonState::BusRechtdoorRechtsaf)),
                    false => out_compat_json_str(self.construct_bulk_json(JsonState::Groen)),
                };
                out_tx.send(json_str).unwrap();
                Some(SignalGroupState::Busy{ start: time })
            },

            SignalGroupState::ForceRed => {
                println!("=> Forced to stop this ControlGroup");
                self.force_red();
                Some(SignalGroupState::Busy{ start:time })
            },

            SignalGroupState::Busy { start } => {
                println!("=> Busy {:?} {:?}", start + self.max_green, self.ids());

                if self.controls_have_state(TrafficLightState::Red) {
                    Some(SignalGroupState::Done)
                }
                else if !self.unlimited_green && time >= start + self.max_green {
                    println!("forcing red...");
                    Some(SignalGroupState::ForceRed)
                }
                else {
                    self.run_loops(time, out_tx, sensor_states);
                    None
                }
            },

            SignalGroupState::Done => {
                println!("=> Controlgroup done");
                None
            }
        }
    }

    pub fn controls_have_state(&self, phase: TrafficLightState) -> bool {
        unsafe {
            let other_state = intrinsics::discriminant_value(&phase);
            self.controls.iter().all(|c| intrinsics::discriminant_value(&c.state) == other_state)
        }
    }

    fn force_red(&mut self) {
        for c in &mut self.controls {
            c.force_red = true;
        }
    }

    fn run_loops(&mut self, time: i32, out_tx: &Sender<String>, sensor_states: &SensorStates) {
        for c in &mut self.controls {
            c.run_loop(time, out_tx, sensor_states, self.unlimited_green);
        }
    }

    fn construct_bulk_json(&self, state: JsonState) -> Vec<StoplichtJson> {
        self.controls.iter().flat_map(|ref c| c.inner.json_objs(state)).collect()
    }

    fn ids(&self) -> Vec<Vec<usize>> {
        self.controls.iter().map(|c| c.inner.get_ids()).collect()
    }
}
