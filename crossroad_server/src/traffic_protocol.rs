use error::{Result, Error, JsonError};
use serde::de;
use serde_json;
use std::fmt;
use time;
use std::collections::HashMap;
use crossroad::*;

pub const BAAN_COUNT: usize = 35; //TODO: REMOVE


// -------------------------------------------------------------------------------
// Client State
// -------------------------------------------------------------------------------

pub struct SensorStates {
    banen: [Sensor; BAAN_COUNT], //TODO: use Vec
}

impl SensorStates {
    pub fn new() -> SensorStates {
        let mut inst = SensorStates { banen: [Sensor::new(); BAAN_COUNT] };
        for i in 0..BAAN_COUNT {  inst.banen[i].id = i; }
        inst
    }

    pub fn _debug_update_directly(&mut self, states: Vec<Sensor>) {
        for state in states.iter() {
            self.banen[state.id].bezet = state.bezet;

            if state.bezet {
                self.banen[state.id].last_update = state.last_update;
            }
        }
    }

    pub fn update(&mut self, banen_json: BanenJson) -> BanenJson {
        for baan in banen_json.banen.iter() {
            self.banen[baan.id].bezet = baan.bezet;

            if baan.bezet {
                self.banen[baan.id].last_update = time::now();
            }
        }
        banen_json
    }

    pub fn update_from_json(&mut self, json_str: &str) -> Result<BanenJson>  {
        json_from_str(&json_str).map(|banen_json| self.update(banen_json))
    }

    // pub fn is_bezet(&self, baan_id: usize) -> bool {
    //     self.banen[baan_id].bezet
    // }

    pub fn active_sensors(&self) -> Vec<&Sensor> {
        self.banen.iter()
            .filter(|b| b.bezet)
            //.map(|x|*x)
            .collect()
    }
}

impl fmt::Debug for SensorStates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for baan in self.banen.iter().filter(|b| b.bezet) {
            try!(write!(f, "[{} bezet] ", baan.id))
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
