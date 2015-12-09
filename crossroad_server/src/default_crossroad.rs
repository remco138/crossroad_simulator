use std::collections::HashMap;

use traffic_protocol::*;
use traffic_controls::*;
use signal_group::*;
use crossroad::*;


pub fn create_traffic_lights() -> TrafficLightsBuilder {
    TrafficLightsBuilder::new(34)
         .set_direction(Direction::North, vec![ 1,6,11,22,24,25,31,33 ])
         .set_direction(Direction::East,  vec![ 2,3,7,12,15,19,28,30 ])
         .set_direction(Direction::South, vec![ 0,4,8,13,17,18,21,23,26,32,34 ])
         .set_direction(Direction::West,  vec![ 5,9,10,14,16,20,27,29 ])
         .set_type(Type::Primary, vec![ 2,3,4,   9,10,11 ])
         .set_type_range(Type::Rest, 17, 34)
}

pub fn create_traffic_controls(traffic_lights: &TrafficLightsBuilder) -> Vec<Control> {
    ControlsBuilder::new(&traffic_lights)
         .add_group(vec![ 2, 3], Direction::East, Type::Primary)
         .add_group(vec![ 9,10], Direction::West, Type::Primary)

         .add_group(vec![17,23, 25,     24,26],  Direction::West,  Type::Rest)
         //.add_group(vec![24,26],         Direction::West,  Type::Rest)

         .add_group(vec![19,20, 28,29,  27,30],  Direction::South, Type::Rest)
        // .add_group(vec![27,30],         Direction::South, Type::Rest)

         .add_group(vec![21,22, 31,34,  32,33],  Direction::East,  Type::Rest)
        // .add_group(vec![32,33],         Direction::East,  Type::Rest)
         .create_controls()
}

pub fn create_crossroad<'a>(traffic_controls: &'a Vec<Control<'a>>) -> Crossroad<'a> {

    let indexed_controls = index_controls(&traffic_controls);

    let road_east_2_3                 = indexed_controls[ 2];
    let road_west_9_10                = indexed_controls[ 9];
    let west_bicycle_and_pedestrain   = indexed_controls[17];
    //let west_inner                    = indexed_controls[24];
    let south_bicycle_and_pedestrain  = indexed_controls[19];
    //let south_inner                   = indexed_controls[27];
    let east_bicycle_and_pedestrain   = indexed_controls[21];
    //let east_inner                    = indexed_controls[32];

    let mut directions = HashMap::new();
    directions.insert(Direction::North,
        XorConflictsGroup::new("noord".to_string(), vec![
            indexed_controls[11].conflicting_with(vec![
                east_bicycle_and_pedestrain
            ]),
            indexed_controls[6].conflicting_with(vec![
                &indexed_controls[8], road_west_9_10,    // noord A
                &indexed_controls[12],                   // noord B
                road_east_2_3,

                south_bicycle_and_pedestrain
            ]),
            indexed_controls[1].conflicting_with(vec![
                &indexed_controls[8], road_west_9_10,    // noord A
                &indexed_controls[12],                   // noord B
                &indexed_controls[13],
                &indexed_controls[5],

                west_bicycle_and_pedestrain
            ]),
        ])
    );

    directions.insert(Direction::East,
        XorConflictsGroup::new("oost".to_string(), vec![
            indexed_controls[7].conflicting_with(vec![
                south_bicycle_and_pedestrain
            ]),
            road_east_2_3.conflicting_with(vec![
                &indexed_controls[5], &indexed_controls[6], // oost A
                &indexed_controls[8],                       // oost B
                &indexed_controls[13],

                west_bicycle_and_pedestrain
            ]),
            indexed_controls[12].conflicting_with(vec![
                &indexed_controls[5], &indexed_controls[6], // oost A
                &indexed_controls[8],                       // oost B
                road_west_9_10,
                &indexed_controls[1]
            ]),
            east_bicycle_and_pedestrain.conflicting_with(vec![
                &indexed_controls[8],
                road_west_9_10,
                &indexed_controls[11],

                // &indexed_controls[15],//bus
                // &indexed_controls[16],//bus
            ]),
        ])
    );

    directions.insert(Direction::South,
        XorConflictsGroup::new("zuid".to_string(), vec![
            indexed_controls[4].conflicting_with(vec![
                west_bicycle_and_pedestrain
            ]),
            indexed_controls[13].conflicting_with(vec![
                &indexed_controls[1], road_east_2_3,    // zuid A
                &indexed_controls[5],                   // zuid B
                &indexed_controls[13]
            ]),
            indexed_controls[8].conflicting_with(vec![
                &indexed_controls[1], road_east_2_3,    // zuid A
                &indexed_controls[5],                   // zuid B
                &indexed_controls[6],
                &indexed_controls[12],

                east_bicycle_and_pedestrain,
            ]),
            south_bicycle_and_pedestrain.conflicting_with(vec![
                &indexed_controls[5],
                &indexed_controls[6],
                &indexed_controls[7],
            ]),
            indexed_controls[0].conflicting_with(vec![])
        ]),
    );

    directions.insert(Direction::West,
        XorConflictsGroup::new("west".to_string(), vec![
            indexed_controls[14].conflicting_with(vec![]),
            road_west_9_10.conflicting_with(vec![
                &indexed_controls[12], &indexed_controls[13], // west A
                &indexed_controls[1],                         // west B
                &indexed_controls[6],

                east_bicycle_and_pedestrain
            ]),
            indexed_controls[5].conflicting_with(vec![
                &indexed_controls[12], &indexed_controls[13], // west A
                &indexed_controls[1],                         // west B
                road_east_2_3,
                &indexed_controls[8],

                south_bicycle_and_pedestrain
            ]),
            west_bicycle_and_pedestrain.conflicting_with(vec![
                &indexed_controls[1],
                road_east_2_3,
                &indexed_controls[4],
                // &indexed_controls[16],//bus
            ]),
        ])
    );

    let primary_traffic = vec![
        road_east_2_3,
        &indexed_controls[4],
        road_west_9_10,
        &indexed_controls[11],
    ];

    Crossroad {
        traffic_controls: indexed_controls.clone(),
        primary_group: SignalGroup::new(primary_traffic.clone(), true),
        primary_traffic: primary_traffic.clone(),
        secondary_traffic: vec![
            &indexed_controls[0],
            &indexed_controls[1],
            &indexed_controls[5],
            &indexed_controls[6],
            &indexed_controls[7],
            &indexed_controls[8],
            &indexed_controls[12],
            &indexed_controls[13],
            &indexed_controls[14],
            &east_bicycle_and_pedestrain,
            &south_bicycle_and_pedestrain,
            &west_bicycle_and_pedestrain,
        ],
        priority_traffic: vec![
            &indexed_controls[15],
            &indexed_controls[16],
        ],
        directions: directions,
    }
}
