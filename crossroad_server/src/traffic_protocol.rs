use error::{Result, Error, JsonError};
use serde::de;
use serde::ser;
use serde_json;
use std::fmt;
use time;
use std::collections::HashMap;
use crossroad::*;
use traffic_controls::*;
use std::sync::mpsc::{channel, Sender, Receiver};
use serde_json::error::Error as SerdeError;

pub const BAAN_COUNT: usize = 35; //TODO: REMOVE


// -------------------------------------------------------------------------------
// Client State
// -------------------------------------------------------------------------------

pub struct SensorStates {
    sensors: [Sensor; BAAN_COUNT], //TODO: use Vec
}

impl SensorStates {
    pub fn new() -> SensorStates {
        let mut inst = SensorStates { sensors: [Sensor::new(); BAAN_COUNT] };
        for i in 0..BAAN_COUNT {  inst.sensors[i].id = i; }
        inst
    }

    pub fn has_active(&self, control: &Control) -> bool {
        control.get_ids().iter().any(|&id| self.sensors[id].bezet)
    }

    pub fn has_any_active(&self, controls: &Vec<&Control>) -> bool {
        controls.iter().any(|c| self.has_active(c))
    }

    pub fn _debug_update_directly(&mut self, states: Vec<Sensor>) {
        for state in states.iter() {
            self.sensors[state.id].bezet = state.bezet;

            if state.bezet {
                self.sensors[state.id].last_update = state.last_update;
            }
        }
    }

    pub fn update(&mut self, banen_json: BanenJson) -> BanenJson {
        for sensor in banen_json.stoplichten.iter() {
            self.sensors[sensor.id].bezet = sensor.bezet;

            if sensor.bezet {
                self.sensors[sensor.id].last_update = time::now();
            }
        }
        banen_json
    }

    pub fn update_from_json(&mut self, json_str: &str) -> Result<BanenJson>  {
        json_from_str(&json_str).map(|banen_json| self.update(banen_json))
    }

    pub fn active_sensors(&self) -> Vec<&Sensor> {
        self.sensors.iter().filter(|b| b.bezet).collect()
    }

    pub fn active_and_longest_waiting(&self) -> Option<(&Sensor, Vec<&Sensor>)> {
        let mut active_sensors = self.active_sensors();

        active_sensors.iter()
            .map(|x|*x)
            .min_by(|b| b.last_update)
            .map(|longest_waiting| {
                active_sensors.retain(|&s| s.id != longest_waiting.id);
                (longest_waiting, active_sensors)
            })
    }
}

impl fmt::Debug for SensorStates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for sensor in self.sensors.iter().filter(|b| b.bezet) {
            try!(write!(f, "[{} bezet] ", sensor.id))
        }
        Ok(())
    }
}


// -------------------------------------------------------------------------------
// Protocol: Client -> Server
// -------------------------------------------------------------------------------

#[derive(Deserialize, Debug, Clone)]
pub struct BanenJson {
    pub stoplichten: Vec<BaanSensorJson>
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct BaanSensorJson {
    pub id: usize,
    pub bezet: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Sensor {
    pub id: usize,
    pub bezet: bool,
    pub last_update: time::Tm,
}

impl Sensor {
    fn new() -> Sensor { Sensor { id: 0, bezet: false, last_update: time::empty_tm() } }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BusBaan {
    pub id: usize,
    pub eerstvolgendelijn: i32,
    pub bezet: bool,
}


// -------------------------------------------------------------------------------
// Protocol: Server -> Client
// -------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum JsonState {
    Rood = 0,
    Geel = 1,
    Groen = 2,
    BusRechtdoorRechtsaf  = 3,
    BusRechtdoor = 4,
    BusRechtsaf = 5
}

impl JsonState {
    pub fn id(&self) -> usize {
        *self as usize
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientJson {
    pub stoplichten: Vec<StoplichtJson>
}

impl ClientJson {
    pub fn from(banen: Vec<StoplichtJson>) -> ClientJson {
        ClientJson { stoplichten: banen }
    }
    pub fn serialize(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct StoplichtJson {
    pub id: usize,
    pub status: usize,
}

impl StoplichtJson {
    pub fn empty() -> StoplichtJson {
        StoplichtJson { id: 0, status: JsonState::Rood.id() }
    }
}

// -------------------------------------------------------------------------------
// Json
// -------------------------------------------------------------------------------
/*
TODO: REMOVE
pub fn send_json(out_tx: &Sender<String>, obj: StoplichtJson) -> serde_json::error::Result<String>  {
    serde_json::to_string(obj).and_then(|json_str| {
        out_tx.send(json_str).unwrap() // TODO: remove unwraps!
    });
}*/

pub fn json_from_str<T>(s: &str) -> Result<T> where T: de::Deserialize {
    serde_json::from_str(&s).map_err(|serd_err| {
        Error::SerdeJson(JsonError::new(&s,  serd_err))
    })
}
