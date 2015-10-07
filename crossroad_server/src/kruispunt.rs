

use traffic_protocol::*;
use error::{Result, Error, JsonError};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use time;


pub enum KruispuntState<'a> {
    Default,
    CreateSignaalGroep,
    SignaalGroep(SignaalGroep<'a>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North, East, South, West
}

pub fn direction_tokens(remove_tokens: &[Direction]) -> Vec<Direction> {
    let v = vec![Direction::North, Direction::East, Direction::South, Direction::West];
    v.iter().filter(|d| remove_tokens.iter().any(|other_d| other_d == *d)).map(|x|*x).collect()
}


pub struct Kruispunt<'a> {
    pub state: KruispuntState<'a>,

    pub stoplichten: Vec<&'a VerkeersRegelaar>, 
    pub hoofdbaan_oost: &'a Hoofdbaan<'a>,
    pub hoofdbaan_west: &'a Hoofdbaan<'a>,
    pub zijbanen: Vec<&'a VerkeersRegelaar>,

    pub richtingen: Richtingen<'a>,
    pub conflict_groepen: Vec<&'a XorStoplichtGroep<'a>>,
}

impl<'a> Kruispunt<'a> {
    pub fn new(stoplicht_all: &'a Vec<Stoplicht>,
               stoplichtcombi_2_3: &'a Hoofdbaan<'a>,
               stoplichtcombi_9_10 : &'a Hoofdbaan<'a>) -> Kruispunt<'a> 
    {
        Kruispunt {
            state: KruispuntState::Default,
            stoplichten: vec![
                &stoplicht_all[0],
                &stoplicht_all[1],
                stoplichtcombi_2_3,
                stoplichtcombi_2_3,
                &stoplicht_all[4],
                &stoplicht_all[5],
                &stoplicht_all[6],
                &stoplicht_all[7],
                &stoplicht_all[8],
                stoplichtcombi_9_10,
                stoplichtcombi_9_10,
                &stoplicht_all[11],
                &stoplicht_all[12],
                &stoplicht_all[13],
                &stoplicht_all[14],
                &stoplicht_all[15],
                &stoplicht_all[16],
            ],
            hoofdbaan_oost: stoplichtcombi_2_3,
            hoofdbaan_west: stoplichtcombi_9_10,
            zijbanen: vec![ 
                &stoplicht_all[0],
                &stoplicht_all[1],
                &stoplicht_all[4],
                &stoplicht_all[5],
                &stoplicht_all[6],
                &stoplicht_all[7],
                &stoplicht_all[8],
                &stoplicht_all[11],
                &stoplicht_all[12],
                &stoplicht_all[13],
                &stoplicht_all[14],
                &stoplicht_all[15],
                &stoplicht_all[16],
            ],
            richtingen: Richtingen::new(&stoplicht_all, &stoplichtcombi_2_3, &stoplichtcombi_9_10),
            conflict_groepen: vec![],
        }
    }
    
    pub fn do_loop(&mut self, tijd: i32, sensor_state: Arc<Mutex<SensorStates>>) -> Option<KruispuntState> {
        let ref mut sensor_state = *sensor_state.lock().unwrap();

        let a = match self.state {

            KruispuntState::Default => {
                // if sensor bezet op 1+ zijbaan
                None
            },

            KruispuntState::CreateSignaalGroep => {
                println!("========== STATUS: CreateSignaalGroep");
                
                let groep = self.get_signaalgroep_from(&sensor_state);
                //let groep = SignaalGroep::empty();

                Some(KruispuntState::SignaalGroep(groep))
            },

            KruispuntState::SignaalGroep(ref groep) => {
                println!("========== STATUS: SignaalGroep");

                groep.do_loop(tijd);

                match groep.status {
                    SignaalGroepStatus::Afgehandeld => {
                        Some(KruispuntState::Default)
                    },
                    _ => None,
                }
            },
        };

        self.state = KruispuntState::Default;

        None
    }

    pub fn get_signaalgroep_from<'b>(&'a self, sensor_state: &'b SensorStates) -> SignaalGroep<'a> {
        
        let start = time::PreciseTime::now();

        // Occupied lanes
        let banen_bezet: Vec<_> = sensor_state.banen_bezet();

        // Longest waiting lane
        let selected_baan: &Baan = banen_bezet.iter().min_by(|b| b.last_update).unwrap();
        let selected_direction = self.richtingen.get_direction_by_id(selected_baan.id).unwrap();
        let start_road = VerkeersRegelaarDirection::new(self.get_obj(selected_baan.id), selected_direction.clone());

        let all_stoplichten: Vec<VerkeersRegelaarDirection> = banen_bezet.iter()
                .filter_map(|b| {
                    self.richtingen.get_direction_by_id(b.id).map(|&d| {
                        VerkeersRegelaarDirection::new(self.get_obj(b.id), d.clone())
                    })
                }).collect();

        
        //let start_road = all_stoplichten.iter().find(|vkd| vkd.has_id(selected_baan.id)).unwrap();
        let conflicted_ids = self.richtingen.get_conflicts_for(&start_road).unwrap();

        // Filter directions
        let available_banen: Vec<_> = all_stoplichten.iter()
            .filter(|vkd| vkd.direction != start_road.direction)
            .filter(|vkd| !vkd.has_ids(&conflicted_ids))
            .map(|x| x.clone()).collect();

        print!("\n\nCalculation done in: {:?}", start.to(time::PreciseTime::now()).num_milliseconds());
        print!("\n\n");

        SignaalGroep::from(start_road, available_banen, &self.richtingen)
    }

    pub fn get_obj(&self, id: usize) -> &'a VerkeersRegelaar {
        self.stoplichten[id]
    }
} 

pub struct Richtingen<'a> {
    pub entries: HashMap<Direction, XorStoplichtGroep<'a>>,
}

impl<'a> Richtingen<'a> {

    pub fn new(stoplicht_all: &'a Vec<Stoplicht>,
               stoplichtcombi_2_3: &'a Hoofdbaan,
               stoplichtcombi_9_10 : &'a Hoofdbaan) -> Richtingen<'a>
    {
        let mut entries = HashMap::new();

        entries.insert(Direction::North, 
            XorStoplichtGroep::from("noord".to_string(), vec![
                stoplicht_all[11].conflicting_with(vec![]),
                stoplicht_all[6].conflicting_with(vec![
                    &stoplicht_all[8], stoplichtcombi_9_10,  // noord A
                    &stoplicht_all[12],                      // noord B
                    stoplichtcombi_2_3,
                ]),
                stoplicht_all[1].conflicting_with(vec![
                    &stoplicht_all[8], stoplichtcombi_9_10,  // noord A
                    &stoplicht_all[12],                      // noord B
                    &stoplicht_all[13],
                    &stoplicht_all[5],
                ]),
            ])
        );

        entries.insert(Direction::East, 
            XorStoplichtGroep::from("oost".to_string(), vec![
                stoplicht_all[7].conflicting_with(vec![]),
                stoplichtcombi_2_3.conflicting_with(vec![
                    &stoplicht_all[5], &stoplicht_all[6],   // oost A
                    &stoplicht_all[8],                      // oost B
                    &stoplicht_all[13]
                ]),
                stoplicht_all[12].conflicting_with(vec![
                    &stoplicht_all[5], &stoplicht_all[6],   // oost A
                    &stoplicht_all[8],                      // oost B
                    stoplichtcombi_9_10,
                    &stoplicht_all[1]
                ]),
            ])
        );

        entries.insert(Direction::South, 
            XorStoplichtGroep::from("zuid".to_string(), vec![
                stoplicht_all[4].conflicting_with(vec![]),
                stoplicht_all[13].conflicting_with(vec![
                    &stoplicht_all[1], stoplichtcombi_2_3,  // zuid A
                    &stoplicht_all[5],                      // zuid B
                    &stoplicht_all[13]
                ]),
                stoplicht_all[8].conflicting_with(vec![
                    &stoplicht_all[1], stoplichtcombi_2_3,  // zuid A
                    &stoplicht_all[5],                      // zuid B
                    &stoplicht_all[6],
                    &stoplicht_all[12]
                ]),
            ])
        );

        entries.insert(Direction::West, 
            XorStoplichtGroep::from("west".to_string(), vec![
                stoplicht_all[14].conflicting_with(vec![]),
                stoplichtcombi_9_10.conflicting_with(vec![
                    &stoplicht_all[12], &stoplicht_all[13], // west A 
                    &stoplicht_all[1],                      // west B
                    &stoplicht_all[6]
                ]),
                stoplicht_all[5].conflicting_with(vec![
                    &stoplicht_all[12], &stoplicht_all[13], // west A
                    &stoplicht_all[1],                      // west B
                    stoplichtcombi_2_3,
                    &stoplicht_all[8]
                ]),
            ])
        ); 

        Richtingen {
            entries: entries
        }  
    }

    pub fn get_direction_by_id(&self, id: usize) -> Option<&Direction> {    
        self.entries.iter().find(|&(k, entry)| entry.conflicts_contains_id(id)).map(|(k, entry)| k)
    }

    pub fn get_conflicts_for(&self, vkd: &VerkeersRegelaarDirection) -> Option<Vec<usize>> {
        self.entries.get(&vkd.direction)
                    .and_then(|xor_groep| xor_groep.get_conflicts_for_id(vkd.get_id()))
    }

    pub fn get_xor_groep(&'a self, direction: &Direction) -> Option<&'a XorStoplichtGroep<'a>> {
        self.entries.get(direction)
    }
}

pub fn generate_all_stoplichten() -> Vec<Stoplicht> {
    (0..BAAN_COUNT).map(|i| Stoplicht::new(i, StoplichtFase::Rood)).collect()
}