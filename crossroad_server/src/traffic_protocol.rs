use error::{Result, Error, JsonError};
use serde::de;
use serde_json;
use std::fmt;
use time;
use std::collections::HashMap;
use kruispunt::*;

pub const BAAN_COUNT: usize = 17;

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


#[derive(Debug)]
pub enum MaxGroenTijd {
    Oneindig,
    Waarde(i32),
}


// -------------------------------------------------------------------------------
// SignaalGroep 
// -------------------------------------------------------------------------------

#[derive(Debug)]
pub enum SignaalGroepStatus {
    Start,
    GarantieGroen   { eind_tijd: i32 },
    Bezig           { eind_tijd: i32 },
    Hiaat           { eind_tijd: i32 },
    Afgehandeld,
}

impl SignaalGroepStatus {
    
}


pub struct SignaalGroep<'a>  {
    pub banen: Vec<&'a VerkeersRegelaar>,
    pub status: SignaalGroepStatus,
}

impl<'a> SignaalGroep<'a> {

    pub fn empty() -> SignaalGroep<'a> {
       SignaalGroep {
           banen: vec![],
           status: SignaalGroepStatus::Start,
        } 
    }

    pub fn from(start_road: VerkeersRegelaarDirection<'a>, entry_options: Vec<VerkeersRegelaarDirection<'a>>, richtingen: &'a Richtingen<'a>) -> SignaalGroep<'a>
    {  
        /*
        let start_entry = XorRealisatie::new(&start_road, &richtingen);
        
        let entries: HashMap<Direction, XorRealisatie<'a>> = HashMap::new();   
        entries.insert(start_road.direction, start_entry);

        //richtingen.get_xor_groep(&selected_direction)
        */

        SignaalGroep {
           banen: vec![ start_road.inner ],
           status: SignaalGroepStatus::Start,
        }
    }

    pub fn do_loop(&self, tijd: i32) -> Option<SignaalGroepStatus> { 

        match self.status {

            SignaalGroepStatus::Start => {
                println!("---Start");

                for baan in self.banen.iter() {
                    println!("---Send {:?} groen licht", baan);
                }

                Some(SignaalGroepStatus::GarantieGroen { eind_tijd: tijd + 5 })
            },

            SignaalGroepStatus::GarantieGroen{ eind_tijd: eind } => {
                println!("---GarantieGroen tijd {:?} > eind: {:?} = {:?}", tijd, eind, tijd >= eind);

                if tijd >= eind { Some(SignaalGroepStatus::Bezig{ eind_tijd: tijd + 7 }) }
                else { None }
            },

            SignaalGroepStatus::Bezig{ eind_tijd: eind } => {
                println!("---Bezig");

                if tijd >= eind { Some(SignaalGroepStatus::Hiaat { eind_tijd: tijd + 7 }) }
                else { None }    
            },

            SignaalGroepStatus::Hiaat{ eind_tijd: eind } => {
                println!("---Hiaat");
                Some(SignaalGroepStatus::Bezig { eind_tijd: tijd + 7})
            },

            SignaalGroepStatus::Afgehandeld => {
                println!("---Afgehandeld");
                None           
            }
        }
    }
}

//
//  Xor 'Richting' Helper structs
//

pub struct XorRealisatie<'a> {
    pub keuze: &'a VerkeersRegelaar,
    pub xor: &'a XorStoplichtGroep<'a>
}

impl<'a> XorRealisatie<'a> {
    pub fn new(keuze: &'a VerkeersRegelaarDirection, richtingen: &'a Richtingen<'a>) -> XorRealisatie<'a> {
        XorRealisatie {
            keuze: keuze,
            xor: richtingen.get_xor_groep(&keuze.direction).unwrap(),
        }
    }
}


#[derive(Debug)]
pub struct XorStoplichtGroep<'a> {
    pub name: String,
    conflicts: Vec<ConflictEntry<'a>>,
}

impl<'a> XorStoplichtGroep<'a> {
    pub fn from(name: String, conflicts: Vec<ConflictEntry<'a>>) -> XorStoplichtGroep {
        XorStoplichtGroep { name: name, conflicts:conflicts }
    }

    pub fn conflicts_contains_id(&'a self, id: usize) -> bool {
        self.conflicts.iter().find(|s| s.regelaar.has_id(id)).is_some()
    }

    pub fn get_conflicts_for_id(&'a self, id: usize) ->  Option<Vec<usize>> { 
        let selected_conflict = self.conflicts.iter().find(|c| c.regelaar.has_id(id));

        selected_conflict.map(|conflict| {
            let mut top_node_ids:Vec<usize> = self.conflicts.iter().map(|cf| cf.regelaar.get_id()).collect();
            let extra_confl_ids = conflict.get_conflict_ids();
            top_node_ids.extend(extra_confl_ids);
            top_node_ids         
        })               
    }
}


#[derive(Debug)]
pub struct ConflictEntry<'a> {
    pub regelaar: &'a VerkeersRegelaar,
    pub conflicting_with: Vec<&'a VerkeersRegelaar>,
}

impl<'a> ConflictEntry<'a> {
    pub fn get_conflict_ids(&self) -> Vec<usize> {
        self.conflicting_with.iter().map(|vr| vr.get_id()).collect()
    }

    pub fn contains(&self, other: &'a VerkeersRegelaar) -> bool {
        self.conflicting_with.iter().find(|x| x.get_id() == other.get_id()).is_some()
    }

    pub fn from(src: &'a ConflictEntry<'a>, extra: Vec<&'a VerkeersRegelaar>) -> ConflictEntry<'a> {
        let mut inst = ConflictEntry { 
            regelaar: src.regelaar,
            conflicting_with: src.conflicting_with.clone(),
        };
        inst.conflicting_with.push_all(extra.as_slice());
        inst
    }
}


// -------------------------------------------------------------------------------
// StoplichtFase / Banen / Hoofdbaan / VerkeersRegelaar
// -------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum StoplichtFase {
    Rood, 
    Geel    { start_time: i32 },
    Groen   { start_time: i32 },
}

pub trait VerkeersRegelaar: fmt::Debug {
    fn conflicting_with<'a>(&'a self, conflicting_with: Vec<&'a VerkeersRegelaar>) -> ConflictEntry where Self: Sized {
        ConflictEntry {
            regelaar: self,
            conflicting_with: conflicting_with,
        }
    }
    fn get_id(&self) -> usize;
    fn has_id(&self, id: usize) -> bool;
    fn has_ids(&self, ids: &Vec<usize>) -> bool;
}

#[derive(Debug, Clone)]
pub struct VerkeersRegelaarDirection<'a> {
    pub inner: &'a VerkeersRegelaar,
    pub direction: Direction,
}

impl<'a> VerkeersRegelaarDirection<'a>  {
    pub fn new(inner: &'a VerkeersRegelaar, direction: Direction) -> VerkeersRegelaarDirection<'a> {
        VerkeersRegelaarDirection { inner:inner, direction: direction }
    }
}

impl <'a>VerkeersRegelaar for VerkeersRegelaarDirection<'a> { 
    fn get_id(&self) -> usize {
        self.inner.get_id()
    }
    fn has_id(&self, id: usize) -> bool {
        self.inner.has_id(id)
    }
    fn has_ids(&self, ids: &Vec<usize>) -> bool {
        self.inner.has_ids(ids)
    }
}


#[derive(Debug)]
pub struct Hoofdbaan<'a> {
    pub baan_1: &'a Stoplicht,
    pub baan_2: &'a Stoplicht,
    pub einde_max_rood_tijd: i32,
    pub fase: StoplichtFase, 
}

impl<'a> Hoofdbaan<'a> {
    pub fn from(baan_1: &'a Stoplicht, baan_2: &'a Stoplicht) -> Hoofdbaan<'a> {
        Hoofdbaan { 
            baan_1:baan_1, 
            baan_2:baan_2,
            einde_max_rood_tijd: i32::max_value(),
            fase: StoplichtFase::Rood,
        }
    }

    pub fn if_moet_op_groen(&self, tijd: &i32) -> bool {
        self.fase == StoplichtFase::Rood && self.einde_max_rood_tijd >= *tijd
    }
}

impl <'a>VerkeersRegelaar for Hoofdbaan<'a> { 
    fn has_id(&self, id: usize) -> bool {
        self.baan_1.id == id || self.baan_2.id == id
    }
    fn get_id(&self) -> usize {
        self.baan_1.id
    }
    fn has_ids(&self, ids: &Vec<usize>) -> bool {
        self.baan_1.has_ids(ids) 
    }
}


#[derive(Debug)]
pub struct Stoplicht {
    pub id: usize,
    pub fase: StoplichtFase,
}

impl Stoplicht {
    pub fn new(id: usize, fase: StoplichtFase) -> Stoplicht {
        Stoplicht { id:id, fase:fase }
    }
}

impl VerkeersRegelaar for Stoplicht { 
    fn get_id(&self) -> usize {
        self.id
    }
    fn has_id(&self, id: usize) -> bool {
        self.id == id
    }
    fn has_ids(&self, ids: &Vec<usize>) -> bool {
        ids.iter().any(|id| *id == self.id)
    }
}


// -------------------------------------------------------------------------------
// Client Side State
// -------------------------------------------------------------------------------


pub struct SensorStates {
    banen: [Baan; BAAN_COUNT],
}

impl SensorStates { 
    pub fn new() -> SensorStates { 
        let mut inst = SensorStates { banen: [Baan::new(); BAAN_COUNT] };
        for i in 0..BAAN_COUNT {  inst.banen[i].id = i; }
        inst
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

    pub fn is_bezet(&self, baan_id: usize) -> bool {
        self.banen[baan_id].bezet
    }

    pub fn banen_bezet(&self, ) -> Vec<&Baan> {
        self.banen.iter()
            .filter(|b| b.bezet)
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
    pub banen: Vec<BaanJson>
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct BaanJson {
    pub id: usize,
    pub bezet: bool,   
}

#[derive(Debug, Copy, Clone)]
pub struct Baan {
    pub id: usize,
    pub bezet: bool,
    pub last_update: time::Tm,  
}

impl Baan {
    fn new() -> Baan { Baan { id: 0, bezet: false, last_update: time::empty_tm() } }
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