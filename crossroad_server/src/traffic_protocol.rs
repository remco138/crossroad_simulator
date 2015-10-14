use error::{Result, Error, JsonError};
use serde::de;
use serde_json;
use std::fmt;
use time;
use std::collections::HashMap;
use crossroad::*;
use traffic_controls::*;

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

    pub fn has_any_active(&self, controls: &Vec<&TrafficControl>) -> bool {
        self.sensors.iter().any(|sensor| {
            controls.iter().any(|control| control.has_id(sensor.id))
        })
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
        for sensor in banen_json.banen.iter() {
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
    pub banen: Vec<BaanSensorJson>
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
pub enum StoplichtJsonStatus {
    Rood = 0,
    Geel = 1,
    Groen = 2,
    BusRechtdoorRechtsaf  = 3,
    BusRechtdoor = 4,
    BusRechtsaf = 5
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoplichtArrayJson {
    pub banen: Vec<StoplichtJson>
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct StoplichtJson {
    pub id: usize,
    pub status: usize,
}

impl StoplichtJson {
    pub fn new() -> StoplichtJson {
        StoplichtJson { id: 0, status: StoplichtJsonStatus::Rood as usize }
    }
}


// -------------------------------------------------------------------------------
// Json
// -------------------------------------------------------------------------------

pub fn json_from_str<T>(s: &str) -> Result<T> where T: de::Deserialize {
    serde_json::from_str(&s).map_err(|serd_err| {
        Error::SerdeJson(JsonError::new(&s,  serd_err))
    })
}
