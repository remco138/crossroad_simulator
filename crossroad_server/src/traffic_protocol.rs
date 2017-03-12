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

#[derive(Debug, Copy, Clone)]
pub struct Sensor {
    pub id: usize,
    pub bezet: bool,
    pub last_update: time::Tm,
}

impl Sensor {
    fn new() -> Sensor { Sensor { id: 0, bezet: false, last_update: time::empty_tm() } }

    pub fn update(&mut self, baan: &Baan) {
        self.bezet = baan.bezet;
        if baan.bezet { self.last_update = time::now(); }
    }

    pub fn update_bus(&mut self, baan: &BusBaan) {
        self.bezet = baan.bezet;
        if baan.bezet { self.last_update = time::now(); }
    }
}

pub struct SensorStates {
    sensors: [Sensor; BAAN_COUNT],
    bus_sensors: [Sensor; BAAN_COUNT],
    current_bus_id: i32,
}

impl SensorStates {

    pub fn new() -> SensorStates {
        let mut inst = SensorStates {
            sensors: [Sensor::new(); BAAN_COUNT],
            bus_sensors: [Sensor::new(); BAAN_COUNT],
            current_bus_id: 0,
        };
        for i in 0..BAAN_COUNT {  inst.sensors[i].id = i; }
        for i in 0..BAAN_COUNT {  inst.bus_sensors[i].id = i; }
        inst
    }


    pub fn has_active(&self, control: &Control) -> bool {
        control.get_ids().iter().any(|&id| self.sensors[id].bezet)
    }
    pub fn has_any_active(&self, controls: &Vec<&Control>) -> bool {
        controls.iter().any(|c| self.has_active(c))
    }


    pub fn has_active_bus(&self, control: &Control) -> bool {
        control.get_ids().iter().any(|&id| self.bus_sensors[id].bezet)
    }
    pub fn has_any_active_bus(&self, controls: &Vec<&Control>) -> bool {
        controls.iter().any(|c| self.has_active_bus(c))
    }


    pub fn update(&mut self, banen: &Vec<Baan>) {
        for baan in banen.iter() {
            self.sensors[baan.id].update(baan);
        }
    }
    pub fn update_bussen(&mut self, busbanen: &Vec<BusBaan>) {
        for baan in busbanen.iter() {
            self.bus_sensors[baan.id].update_bus(baan);

            if baan.bezet {
                self.current_bus_id = baan.eerstvolgendelijn;
            }
        }
    }

    pub fn active_sensors(&self) -> Vec<&Sensor> {
        self.sensors.iter().filter(|b| b.bezet).collect()
    }

    pub fn active_and_longest_waiting(&self) -> Option<(&Sensor, Vec<&Sensor>)> {
        let mut active_sensors = self.active_sensors();

// TODOFIX
        // active_sensors.iter()
        //     .map(|x|*x)
        //     .min_by(|b| b.last_update)
        //     .map(|longest_waiting| {
        //         active_sensors.retain(|&s| s.id != longest_waiting.id);
        //         (longest_waiting, active_sensors)
        //     })
        None
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
// Protocol: All in 1 Json
// -------------------------------------------------------------------------------

pub static mut JSON_COMPAT_LEVEL: JsonCompatLevel = JsonCompatLevel::None; //plsnostatic :|

pub enum JsonCompatLevel {
    None, Null, Empty,
}

impl JsonCompatLevel {
    pub fn from_str(s: &str) -> Option<JsonCompatLevel> {
        match s {
            "none"  => Some(JsonCompatLevel::None),
            "null"  => Some(JsonCompatLevel::Null),
            "empty" => Some(JsonCompatLevel::Empty),
            _       => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolJson {
    pub banen: Option<Vec<Baan>>,
    pub busbanen: Option<Vec<BusBaan>>,
    pub stoplichten: Option<Vec<StoplichtJson>>,
}

impl ProtocolJson {
    pub fn vec_is_null(c: ClientJson) -> ProtocolJson {
        ProtocolJson {
            banen: None,
            busbanen: None,
            stoplichten: Some(c.stoplichten),
        }
    }
    pub fn vec_is_empty(c: ClientJson) -> ProtocolJson {
        ProtocolJson {
            banen: Some(vec![]),
            busbanen: Some(vec![]),
            stoplichten: Some(c.stoplichten),
        }
    }
}

pub fn out_compat_json_str(stoplichten: Vec<StoplichtJson>) -> String {
    let json_obj = ClientJson::new(stoplichten);

    unsafe {
        match JSON_COMPAT_LEVEL {
            JsonCompatLevel::None  => serde_json::to_string(&json_obj),
            JsonCompatLevel::Null  => serde_json::to_string(&ProtocolJson::vec_is_null(json_obj)),
            JsonCompatLevel::Empty => serde_json::to_string(&ProtocolJson::vec_is_empty(json_obj)),
        }.unwrap()
    }
}

// -------------------------------------------------------------------------------
// Protocol: Client -> Server
// -------------------------------------------------------------------------------

#[derive(Deserialize, Debug, Clone)]
pub struct BanenJson {
    pub banen: Vec<Baan>
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Baan {
    pub id: usize,
    pub bezet: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub fn new(stoplichten: Vec<StoplichtJson>) -> ClientJson {
        ClientJson { stoplichten: stoplichten }
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
