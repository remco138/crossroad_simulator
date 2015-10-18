use traffic_protocol::*;
use traffic_controls::*;
use signal_group::*;
use error::{Result, Error, JsonError};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};
use time;
use cartesian;

// -------------------------------------------------------------------------------
// Direction
// -------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
	North, East, South, West
}

impl Direction {
	pub fn all() -> Vec<Direction> {
		vec![Direction::North, Direction::East, Direction::South,  Direction::West]
	}
	pub fn retain<'a, 'b, 'c>(&self, src: &Vec<&'c ControlSensor<'a, 'b>>) -> Vec<&'c ControlSensor<'a, 'b>> {
		let mut vec = src.clone();
		vec.retain(|c| c.inner.direction() == *self);
		vec
	}
}

pub fn directions() -> Vec<Direction> {
	vec![Direction::North, Direction::East, Direction::South, Direction::West]
}

// -------------------------------------------------------------------------------
// Crossroad
// -------------------------------------------------------------------------------

pub enum CrossroadState<'a> {
	AllRed,
	PrimaryTraffic(SignalGroup<'a>),
	CreatePriorityGroup,
	CreateSignalGroup,
	SignalGroup(SignalGroup<'a>),
}

pub struct Crossroad<'a> {
	traffic_controls: Vec<&'a Control<'a>>,
	pub primary_group: SignalGroup<'a>,
	pub primary_traffic: Vec<&'a Control<'a>>,
	pub secondary_traffic: Vec<&'a Control<'a>>,
	pub priority_traffic: Vec<&'a Control<'a>>,
	pub directions: HashMap<Direction, XorConflictsGroup<'a>>
}

impl<'a> Crossroad<'a> {

	pub fn new(traffic_lights: &'a Vec<Control<'a>>,
			   primary_road_east_2_3: &'a Control<'a>,
			   primary_road_west_9_10 : &'a Control<'a>,
			   crossing_east:  &'a Crossing<'a>,
			   crossing_south: &'a Crossing<'a>,
			   crossing_west:  &'a Crossing<'a>,)
			-> Crossroad<'a> {

		let mut directions = HashMap::new();

		directions.insert(Direction::North,
			XorConflictsGroup::new("noord".to_string(), vec![
				traffic_lights[11].conflicting_with(vec![]),
				traffic_lights[6].conflicting_with(vec![
					&traffic_lights[8], primary_road_west_9_10,  // noord A
					&traffic_lights[12],                      // noord B
					primary_road_east_2_3,
				]),
				traffic_lights[1].conflicting_with(vec![
					&traffic_lights[8], primary_road_west_9_10,  // noord A
					&traffic_lights[12],                         // noord B
					&traffic_lights[13],
					&traffic_lights[5],
				]),
			])
		);

		directions.insert(Direction::East,
			XorConflictsGroup::new("oost".to_string(), vec![
				traffic_lights[7].conflicting_with(vec![]),
				primary_road_east_2_3.conflicting_with(vec![
					&traffic_lights[5], &traffic_lights[6],   // oost A
					&traffic_lights[8],                       // oost B
					&traffic_lights[13]
				]),
				traffic_lights[12].conflicting_with(vec![
					&traffic_lights[5], &traffic_lights[6],   // oost A
					&traffic_lights[8],                       // oost B
					primary_road_west_9_10,
					&traffic_lights[1]
				]),
				crossing_east.bicycle_and_pedestrain.conflicting_with(vec![
					&traffic_lights[8],
					primary_road_west_9_10,
					&traffic_lights[11],
				]),
			])
		);

		directions.insert(Direction::South,
			XorConflictsGroup::new("zuid".to_string(), vec![
				traffic_lights[4].conflicting_with(vec![]),
				traffic_lights[13].conflicting_with(vec![
					&traffic_lights[1], primary_road_east_2_3,  // zuid A
					&traffic_lights[5],                         // zuid B
					&traffic_lights[13]
				]),
				traffic_lights[8].conflicting_with(vec![
					&traffic_lights[1], primary_road_east_2_3,  // zuid A
					&traffic_lights[5],                         // zuid B
					&traffic_lights[6],
					&traffic_lights[12]
				]),
				crossing_south.bicycle_and_pedestrain.conflicting_with(vec![
					&traffic_lights[5],
					&traffic_lights[6],
					&traffic_lights[7],
				]),
			]),
		);

		directions.insert(Direction::West,
			XorConflictsGroup::new("west".to_string(), vec![
				traffic_lights[14].conflicting_with(vec![]),
				primary_road_west_9_10.conflicting_with(vec![
					&traffic_lights[12], &traffic_lights[13],   // west A
					&traffic_lights[1],                         // west B
					&traffic_lights[6]
				]),
				traffic_lights[5].conflicting_with(vec![
					&traffic_lights[12], &traffic_lights[13],   // west A
					&traffic_lights[1],                         // west B
					primary_road_east_2_3,
					&traffic_lights[8]
				]),
				crossing_west.bicycle_and_pedestrain.conflicting_with(vec![
					&traffic_lights[1],
					primary_road_east_2_3,
					&traffic_lights[4],
				]),
			])
		);

		Crossroad {
			traffic_controls: vec![
				&traffic_lights[0],
				&traffic_lights[1],
				primary_road_east_2_3,
				primary_road_east_2_3,
				&traffic_lights[4],
				&traffic_lights[5],
				&traffic_lights[6],
				&traffic_lights[7],
				&traffic_lights[8],
				primary_road_west_9_10,
				primary_road_west_9_10,
				&traffic_lights[11],
				&traffic_lights[12],
				&traffic_lights[13],
				&traffic_lights[14],
				&traffic_lights[15],
				&traffic_lights[16],
				&crossing_west.bicycle_and_pedestrain,
				&traffic_lights[18],    // does not exist !
				&crossing_south.bicycle_and_pedestrain,  // 19
				&crossing_south.bicycle_and_pedestrain,  // 20
				&crossing_east.bicycle_and_pedestrain,   // 21
				&crossing_east.bicycle_and_pedestrain,   // 22
				&crossing_west.bicycle_and_pedestrain,   // 23
				&traffic_lights[24],    // inner pedestrian west
				&crossing_west.bicycle_and_pedestrain,
				&traffic_lights[26],    // inner pedestrian west
				&traffic_lights[27],    // inner pedestrian south
				&crossing_south.bicycle_and_pedestrain,
				&crossing_south.bicycle_and_pedestrain,
				&traffic_lights[30],    // inner pedestrian south
				&crossing_east.bicycle_and_pedestrain,
				&crossing_east.bicycle_and_pedestrain,
				&traffic_lights[33],    // inner pedestrian east
				&traffic_lights[34],    // inner pedestrian east
			],
			primary_group: SignalGroup::new(vec![
				primary_road_east_2_3,
				&traffic_lights[4],
				primary_road_west_9_10,
				&traffic_lights[11]],
				true
			),
			primary_traffic: vec![
				primary_road_east_2_3,
				&traffic_lights[4],
				primary_road_west_9_10,
				&traffic_lights[11],
			],
			secondary_traffic: vec![
				&traffic_lights[0],
				&traffic_lights[1],
				&traffic_lights[5],
				&traffic_lights[6],
				&traffic_lights[7],
				&traffic_lights[8],
				&traffic_lights[12],
				&traffic_lights[13],
				&traffic_lights[14],
				&crossing_east.bicycle_and_pedestrain,
				&crossing_south.bicycle_and_pedestrain,
				&crossing_west.bicycle_and_pedestrain,
			],
			priority_traffic: vec![
				&crossing_east.inner_pedestrian,
				&crossing_south.inner_pedestrian,
				&crossing_west.inner_pedestrian,
				&traffic_lights[15],
				&traffic_lights[16],
			],
			directions: directions,
		}
	}

	pub fn run_loop(&'a self, time: i32,
							  state: &mut CrossroadState<'a>,
							  sensor_shared_state: Arc<Mutex<SensorStates>>,
							  out_tx: &Sender<String>)
						   -> Option<CrossroadState<'a>> {


		let ref mut sensor_states = *sensor_shared_state.lock().unwrap();

		match *state {

			CrossroadState::AllRed => {
				println!("========== STATE: AllRed");

				if sensor_states.has_any_active(&self.priority_traffic) {
					Some(CrossroadState::CreatePriorityGroup)
				}
				else if sensor_states.has_any_active(&self.secondary_traffic) {
					Some(CrossroadState::CreateSignalGroup)
				}
				else {
					Some(CrossroadState::PrimaryTraffic(self.primary_group.clone()))
				}
			},

			CrossroadState::PrimaryTraffic(ref mut group) => {
				print!("========== STATE: PrimaryTraffic ");
				let group_is_green = group.controls_have_state(TrafficLightState::Green{start:0});
				let any_sensor_active = sensor_states.has_any_active(&self.secondary_traffic) || sensor_states.has_any_active(&self.priority_traffic);

				if group_is_green && any_sensor_active {
					println!(" :)))) Secondary traffic detected. Closing main traffic lanes");
					Some(CrossroadState::SignalGroup(group.clone_with(SignalGroupState::ForceRed)))
				}
				else {
					match group.run_loop(time, out_tx, &sensor_states) {
						Some(SignalGroupState::Done) => Some(CrossroadState::AllRed),
						Some(v) => Some(CrossroadState::PrimaryTraffic(group.clone_with(v))),
						None => None,
					}
				}
			},

			CrossroadState::CreatePriorityGroup => {
				println!("========== STATE: CreatePriorityGroup");
				None
			},

			CrossroadState::CreateSignalGroup => {
				println!("========== STATE: CreateSignalGroup");
				let group = self.generate_signalgroup(&sensor_states);
				Some(CrossroadState::SignalGroup(group))
			},

			CrossroadState::SignalGroup(ref mut group) => {
				print!("========== STATE: SignalGroup ");
				match group.run_loop(time, out_tx, &sensor_states) {
					Some(SignalGroupState::Done) => Some(CrossroadState::AllRed),
					Some(v) => Some(CrossroadState::SignalGroup(group.clone_with(v))),
					None => None,
				}
			}
		}
	}

	pub fn generate_signalgroup(&'a self, sensor_states: &SensorStates) -> SignalGroup<'a> {
		let start = time::PreciseTime::now();
		//
		//
		let (longest_waiting, other_active_sensors) = sensor_states.active_and_longest_waiting().expect("massive boner get_sensor_control");

		let start_control = self.get_sensor_control(longest_waiting).expect("generate_signalgroup get_sensor_control");
		let active_controls = self.get_sensor_controls(&other_active_sensors);

		print!("Start sensor:\n  {:?}\nActive sensors:\n  ", start_control);
		for c in &active_controls { print!("{:?}\n  ", c) };

		let compatible_controls = self.choose_compatible(&start_control, &active_controls);
		let signal_group = self.fill_signal_group(&start_control, &compatible_controls);

		println!("\nFinal group\n {:?}", signal_group);
		//
		//
		print!("\nCalculation done in: {:?} milliseconds", start.to(time::PreciseTime::now()).num_milliseconds());
		print!("\n");

		signal_group
	}

	fn choose_compatible<'b>(&'a self, control: &ControlSensor<'a, 'b>,
									   choices: &Vec<ControlSensor<'a, 'b>>)
									-> Option<Vec<ControlSensor<'a, 'b>>> {

		let until_now = time::now();

		let non_conflicting = control.filter_conflicting(choices);

		let rest_choices: Vec<Vec<&ControlSensor>> = directions().iter()
					.map(|d| d.retain(&non_conflicting))
					.filter(|v| v.len() != 0).collect();

		println!("\nAfter conflicting filter: {:?}", control.conflicting_ids);
		for v in &non_conflicting { println!("  {:?}", v) };
		println!("\nRetained direction vecs");
		for v in &rest_choices { println!("{:?}", v) };

		let mut path_results = vec![];
		for control_path in cartesian::all_possibilities(rest_choices) {
			let mut conflicts = control.conflicting_ids.clone();
			let mut path = vec![];
			let mut acc = time::Duration::zero();

			print!("\nchecking control path: ");
			for c in &control_path { print!("{:?} ", c.inner.get_ids()) };

			for current_control in &control_path {

				match current_control.inner.contains_one_of(&conflicts) {
					true => {
						print!("\n{:?} is conflicting, ignore", current_control.inner.get_ids())
					},
					false => {
						acc = acc + current_control.time_waiting(until_now);
						conflicts.push_all(current_control.conflicting_ids.as_slice());
						path.push(current_control.clone());
						print!("\n{:?} added. +{:?} seconds. New conflicts: {:?} ", current_control.inner.get_ids(), acc.num_seconds(), conflicts)
					}
				}
			}

			path_results.push((path.clone(), acc));
		 };

		 println!("\n\nPath Results:");

		 for &(ref path, count) in &path_results {
			 print!("Option: Combined waiting time = {:?} seconds for the traffic lights path: ", count.num_seconds());
			 for c in path { print!("{:?} ", c.inner.get_ids()) };
			 print!("\n");
		 }

		 path_results.iter()
			.max_by(|&&(ref path, count)| count)
			.map(|&(ref path, count)| {
				path.iter().map(|&c| c.clone()).collect()
			})
	}

	fn fill_signal_group<'b>(&'a self, control: &ControlSensor<'a, 'b>,
									   compatibles: &Option<Vec<ControlSensor<'a, 'b>>>)
								   ->  SignalGroup<'a> {

		let mut traffic_controls = vec![control.inner];

		if let &Some(ref compatible_controls) = compatibles {
			let inners: Vec<_> = compatible_controls.iter().map(|c| c.inner).collect();
			traffic_controls.push_all(inners.as_slice());
		}

		SignalGroup::new(traffic_controls, false)
	}

	pub fn get_sensor_control<'b>(&'a self, sensor: &'b Sensor) -> Option<ControlSensor<'a, 'b>> {
		self.get_traffic_control(sensor.id).map(|control| {
			ControlSensor::new(control, sensor, self.conflicts_for(control))
		})
	}

	pub fn get_sensor_controls<'b>(&'a self, sensors: &Vec<&'b Sensor>) -> Vec<ControlSensor<'a, 'b>> {
		sensors.iter().filter_map(|sensor| {
		   self.get_traffic_control(sensor.id).map(|control| {
			   ControlSensor::new(control, sensor, self.conflicts_for(control))
		   })
		}).collect()
	}

	pub fn conflicts_for(&'a self, control: &Control<'a>) -> Vec<usize> {
		//println!("Getting conflicts for: {:?}", control);
		self.directions
			.get(&control.direction())
			.and_then(|xor| xor.get_conflicts_for(control))
			.expect("the world is ending conflicts_for")
	}

	pub fn get_traffic_control_unsafe(&'a self, id: usize) -> &'a Control {
		&*self.traffic_controls[id]
	}

	pub fn get_traffic_control(&'a self, id: usize) -> Option<&'a Control> {
		self.traffic_controls.get(id).map(|c|*c)
	}
}

pub fn generate_traffic_lights() -> Vec<TrafficLight> {
	vec![
		// roads
		TrafficLight::new(0, Direction::North, Type::Vehicle),

		TrafficLight::new(1, Direction::North,  Type::Vehicle),
		TrafficLight::new(2, Direction::East,   Type::Primary),
		TrafficLight::new(3, Direction::East,   Type::Primary),
		TrafficLight::new(4, Direction::South,  Type::Primary),

		TrafficLight::new(5, Direction::West,   Type::Vehicle),
		TrafficLight::new(6, Direction::North,  Type::Vehicle),
		TrafficLight::new(7, Direction::East,   Type::Vehicle),

		TrafficLight::new(8, Direction::South,  Type::Vehicle),
		TrafficLight::new(9, Direction::West,   Type::Primary),
		TrafficLight::new(10, Direction::West,  Type::Primary),
		TrafficLight::new(11, Direction::North, Type::Primary),

		TrafficLight::new(12, Direction::East,  Type::Vehicle),
		TrafficLight::new(13, Direction::South, Type::Vehicle),
		TrafficLight::new(14, Direction::West,  Type::Vehicle),

		// bus
		TrafficLight::new(15, Direction::East,  Type::Vehicle),
		TrafficLight::new(16, Direction::West,  Type::Vehicle),

		// bycicle
		TrafficLight::new(17, Direction::South, Type::Rest),
		TrafficLight::new(18, Direction::South, Type::Rest),

		TrafficLight::new(19, Direction::East, Type::Rest),
		TrafficLight::new(20, Direction::West, Type::Rest),

		TrafficLight::new(21, Direction::South, Type::Rest),
		TrafficLight::new(22, Direction::North, Type::Rest),

		// pedestrian
		TrafficLight::new(23, Direction::South, Type::Rest),
		TrafficLight::new(24, Direction::North, Type::Rest),
		TrafficLight::new(25, Direction::North, Type::Rest),
		TrafficLight::new(26, Direction::South, Type::Rest),

		TrafficLight::new(27, Direction::West, Type::Rest),
		TrafficLight::new(28, Direction::East, Type::Rest),
		TrafficLight::new(29, Direction::West, Type::Rest),
		TrafficLight::new(30, Direction::East, Type::Rest),

		TrafficLight::new(31, Direction::North, Type::Rest),
		TrafficLight::new(32, Direction::South, Type::Rest),
		TrafficLight::new(33, Direction::North, Type::Rest),
		TrafficLight::new(34, Direction::South, Type::Rest),
	]
}

pub fn to_controls(v: &Vec<TrafficLight>) -> Vec<Control> {
	v.iter().map(|tl| Control::Single(tl)).collect()
}
