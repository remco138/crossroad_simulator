use error::{Result, Error, JsonError};
use serde::de;
use serde_json;
use std::fmt;

const BAAN_COUNT: usize = 17;

// -------------------------------------------------------------------------------
// Server State
// -------------------------------------------------------------------------------

const HIAAT_TIJD: i32 = 5;

const GARANTIE_ROOD_TIJD: i32 = 2;
trait Geelfase { fn geelfase_lengte(&self) -> i32; }
enum Groenfase { Gemotoriseerd, BussenTrams, Fietsers, Voetgangers, }
impl Groenfase {
	fn tijd(&self) -> i32 {
		match *self {
			Groenfase::Gemotoriseerd => 7, // 6 - 8 seconden
			Groenfase::BussenTrams   => 5, // 4 - 6 seconden
			Groenfase::Fietsers      => 7, // 5 - 8 seconden
			Groenfase::Voetgangers   => 5, // 4 - 6 seconden
		}
	}	
}


pub enum MaxGroenTijd {
	Oneindig,
	Waarde(i32),
}

pub enum StoplichtFase {
	Rood, 
	Geel 	{ start_time: i32 },
	Groen 	{ start_time: i32 },
}

pub struct Stoplicht {
	pub id: usize,
	pub fase: StoplichtFase,
	pub max_groentijd: MaxGroenTijd,
}

pub struct StopLichtGroep<'a> {
	pub naam: String,
	pub stoplichten: Vec<&'a Stoplicht>,
	pub max_groentijd: MaxGroenTijd,	
}

pub enum VerkeersRegelEntity<'a> {
	Stoplicht(Stoplicht),
	Groep(StopLichtGroep<'a>),
}

pub struct XorStoplichtGroep<'a> {
	pub naam: String,
	pub stoplichten: Vec<StoplichtConflicting<'a>>,
}

pub struct StoplichtConflicting<'a> {
	pub stoplicht: &'a VerkeersRegelEntity<'a>,
	pub conflicting_with: Vec<&'a VerkeersRegelEntity<'a>>,
}


// -------------------------------------------------------------------------------
// Client Side State
// -------------------------------------------------------------------------------

pub struct BaanSensorStates {
	banen: [Baan; BAAN_COUNT],
}

impl BaanSensorStates {	
	pub fn new() -> BaanSensorStates { 
		let mut inst = BaanSensorStates { banen: [Baan::new(); BAAN_COUNT], };
		for i in 0..BAAN_COUNT {
			inst.banen[i].id = i;
		}
		inst
	}

	pub fn update(&mut self, baan: Baan) -> Baan {
		self.banen[baan.id].bezet = baan.bezet;
		baan
	}

	pub fn update_from_json(&mut self, json_str: &str) -> Result<Baan>  {
        json_from_str(&json_str).map(|baan| self.update(baan))
	}
}



// -------------------------------------------------------------------------------
// Protocol: Client -> Server
// -------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Baan {
	pub id: usize,
	pub bezet: bool,	
}

impl Baan {
	fn new() -> Baan {
		Baan { id: 0, bezet: false }
	}
}

impl fmt::Display for Baan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "baan: {} {}", self.id, self.bezet)
    }
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
	Rood,
	Geel,
	Groen,
	BusRechtdoorRechtsaf,
	BusRechtdoor,
	BusRechtsaf
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct StoplichtJson {
	pub id: usize,
	pub status: StoplichtJsonStatus,
}

impl StoplichtJson {
	fn new() -> StoplichtJson {
		StoplichtJson { id: 0, status: StoplichtJsonStatus::Rood }
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

