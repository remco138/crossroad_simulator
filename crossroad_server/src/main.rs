#![allow(dead_code, unused_variables, unused_imports, unused_must_use,)]

extern crate time;
extern crate serde;
extern crate serde_json;
extern crate schedule_recv;
extern crate crossroad_server; // Local crate

use serde::ser;
use schedule_recv as sched;
use time::*;

use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::fmt::Display;
use std::io::{self, BufRead, Write, BufReader, BufWriter};
use std::thread;
use std::thread::{JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

use crossroad_server::crossroad::*;
use crossroad_server::traffic_protocol::*;
use crossroad_server::traffic_controls::*;
use crossroad_server::error::{Result, Error, JsonError};


fn main() {
    run_server("127.0.0.1:9990").unwrap();
}

fn run_server<A>(address: A) -> io::Result<()> where A: ToSocketAddrs + Display {

    let listener = try!(TcpListener::bind(&address));
    println!("Server listening on: {}", address);

    // Infinite loop.
    for tcp_stream in listener.incoming().filter_map(|i| i.ok()) {
        thread::spawn(move || {
            println!("Connecting a new client");

            match handle_client(tcp_stream) {
                Ok(_) => println!("Client disconnected normally."),
                Err(v) => println!("Client error {:?}", v),
            };
        });
    }

    Ok(())
}

fn handle_client(client_stream: TcpStream) -> io::Result<()> {

    // Convert stream to buffered streams
    let client_reader = BufReader::new(try!(client_stream.try_clone()));
    let client_writer = BufWriter::new(client_stream);

    // Main thread uses this channel to send (json) updates to the client.
    let (out_transmitter, out_receiver) = channel::<String>();
    let (exit_main_loop_tx, exit_main_loop_rx) = channel();

    // Getting updates from the simulator(client) via a socket, so make it safe with reference counter + a mutex.
    let client_baan_sensor_states = Arc::new(Mutex::new(SensorStates::new()));

    // Run seperate threads
    let client_receiver_handle = spawn_client_sensor_receiver(client_reader, client_baan_sensor_states.clone());
    let client_updater_handle = spawn_client_updater(client_writer, out_receiver);
    let verkeersregelinstallatie_handle = spawn_main_loop(out_transmitter, exit_main_loop_rx, client_baan_sensor_states.clone());

    println!("Connection established");

    // Wait for threads to exit
    if let Err(v) = client_updater_handle.join().and(client_receiver_handle.join()) {
        println!("client disconnected, error {:?}", v);
    }

    exit_main_loop_tx.send(0);
    verkeersregelinstallatie_handle.join();
    Ok(())
}

fn spawn_main_loop( out_tx: Sender<String>,
                    exit_rx: Receiver<u8>,
                    sensor_shared_state: Arc<Mutex<SensorStates>>)
                    -> JoinHandle<Result<()>>
 {
    thread::spawn(move || {

        let traffic_lights = generate_traffic_lights();
        let traffic_light_controls = to_controls(&traffic_lights);

        let primary_road_east_2_3 = &TrafficGroup::from(vec![&traffic_lights[2], &traffic_lights[3]], Direction::East, Type::Primary);
        let primary_road_east_2_3_g = Control::Group(primary_road_east_2_3);

        let primary_road_west_9_10 = &TrafficGroup::from(vec![&traffic_lights[9], &traffic_lights[10]], Direction::West, Type::Primary);
        let primary_road_west_9_10_g = Control::Group(primary_road_west_9_10);

        let west_inner = &TrafficGroup::with(vec![&traffic_lights[24], &traffic_lights[26]], Type::Rest);
        let west_byc_ped = &TrafficGroup::from(
            vec![&traffic_lights[17], &traffic_lights[23], &traffic_lights[25]],
            Direction::West,
            Type::Rest
        );

        let south_inner = &TrafficGroup::with(vec![&traffic_lights[27], &traffic_lights[30]], Type::Rest);
        let south_byc_ped = &TrafficGroup::from(
            vec![&traffic_lights[19], &traffic_lights[20],   // bycicle
                 &traffic_lights[28], &traffic_lights[29]],  // pedestrian
             Direction::South,
             Type::Rest
        );

        let east_inner = &TrafficGroup::with(vec![&traffic_lights[32], &traffic_lights[33]], Type::Rest);
        let east_byc_ped = &TrafficGroup::from(
            vec![&traffic_lights[21], &traffic_lights[22],   // bycicle
                 &traffic_lights[31], &traffic_lights[34]],  // pedestrian
             Direction::East,
             Type::Rest
        );

        let crossing_east = Crossing::from(Control::Group(east_byc_ped), Control::Group(east_inner), Direction::East);
        let crossing_south = Crossing::from(Control::Group(south_byc_ped), Control::Group(south_inner), Direction::South);
        let crossing_west = Crossing::from(Control::Group(west_byc_ped), Control::Group(west_inner), Direction::West);

        let crossroad = Crossroad::new(
            &traffic_light_controls,
            &primary_road_east_2_3_g,
            &primary_road_west_9_10_g,
            &crossing_east,
            &crossing_south,
            &crossing_west
        );
        let mut crossroad_state = CrossroadState::AllRed;
        let mut time = 0; // in seconden

        let frequency_scheduler = sched::periodic_ms(1000);

        loop {
            time = time + 1;
            frequency_scheduler.recv().unwrap();
            if let Ok(exit_loop) = exit_rx.try_recv() {
                break;
            }

            print!("\n     {:?} ", time);

            match crossroad.run_loop(time, &mut crossroad_state, sensor_shared_state.clone(), &out_tx) {
                Some(newstate) => crossroad_state = newstate,
                None => (),
            };
        }

        Ok(())
    })
}

fn spawn_client_sensor_receiver(mut reader: BufReader<TcpStream>, sensor_data: Arc<Mutex<SensorStates>>) -> JoinHandle<Result<()>> {
    thread::spawn(move || {
        loop {
            let mut line = String::new();
            try!(reader.read_line(&mut line));

            let line_trimmed = &line[0..(line.len()-1)];
            let ref mut traffic_state = *sensor_data.lock().unwrap();

            match traffic_state.update_from_json(line_trimmed) {
                Ok(banen_json) => println!("Client->Server: received baan sensor update: {:?}\nnew_state = {:?}", banen_json, traffic_state),
                Err(err) => println!("Client->Server: received faulty json string {:?}", line_trimmed),
            }
        }
    })
}

fn spawn_client_updater(mut writer: BufWriter<TcpStream>, rx: Receiver<String>) -> thread::JoinHandle<Result<()>> {
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(msg) => {
                    try!(writer.write(&msg.as_bytes()));
                    try!(writer.flush());
                    println!("Server->Client: sent new stoplicht state {:?}", msg);
                },
                Err(err) => {
                    println!("{:?}", err);
                    return Ok(());
                }
            };
        }
    })
}

#[test]
fn send_json() {
    let stoplicht = &StoplichtJson::empty();
    let stringified = serde_json::to_string(stoplicht).unwrap();

    println!("before:\n{:?}\n\nafter:\n{:?}", stoplicht, stringified);
}

#[test]
fn main_loop() {

    let (out_transmitter, out_receiver) = channel::<String>();
    let (exit_main_loop_tx, exit_main_loop_rx) = channel();
    let client_baan_sensor_states = Arc::new(Mutex::new(SensorStates::new()));

    let sensor_shared_state = client_baan_sensor_states.clone();
    {
        let ref mut init_state = *sensor_shared_state.lock().unwrap();

        let now = time::now();

        init_state._debug_update_directly(vec![
            /*
            Sensor { id: 6,  bezet: true, last_update: now - Duration::seconds(1000) },
            Sensor { id: 5,  bezet: true, last_update: now - Duration::seconds(5) },
            Sensor { id: 11, bezet: true, last_update: now - Duration::seconds(10) },
            Sensor { id: 4,  bezet: true, last_update: now - Duration::seconds(50) },
            Sensor { id: 9,  bezet: true, last_update: now - Duration::seconds(14) },
            Sensor { id: 7,  bezet: true, last_update: now - Duration::seconds(5) },
            Sensor { id: 13, bezet: true, last_update: now - Duration::seconds(6) },*/
        ]);
    }

    let sensor_shared_state_copy = client_baan_sensor_states.clone();
    thread::spawn(move || {

        thread::sleep_ms(13000);
        {
            println!("Sending simulated state #1: sensor 13 = true");
            let ref mut traffic_state = *sensor_shared_state_copy.lock().unwrap();
            traffic_state._debug_update_directly(vec![Sensor { id: 13, bezet: true, last_update: time::now() },]);
        }

        thread::sleep_ms(16000);
        {
            println!("Sending simulated state #2: sensor 13 = false");
            let ref mut traffic_state = *sensor_shared_state_copy.lock().unwrap();
            traffic_state._debug_update_directly(vec![Sensor { id: 13, bezet: false, last_update: time::now() },]);
        }

        thread::sleep_ms(25000);
        {
            println!("Sending simulated state #3: sensor 9 = true");
            let ref mut traffic_state = *sensor_shared_state_copy.lock().unwrap();
            traffic_state._debug_update_directly(vec![Sensor { id: 9, bezet: true, last_update: time::now() },]);
        }
    });

    let verkeer = spawn_main_loop(out_transmitter, exit_main_loop_rx, client_baan_sensor_states.clone());

    loop {
        match out_receiver.recv() {
            Ok(msg) => {
                println!("Server->Client: sent new stoplicht state {:?}", msg)
            },
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        };
    }
}

#[test]
fn time_max() {

    let mut a = vec![];

    a.push(time::now());
    println!("{:?}", a);

    thread::sleep_ms(2000);

    a.push(time::now());
    println!("{:?}", a);

    thread::sleep_ms(2000);

    a.push(time::now());
    println!("{:?}", a);

    let b = a.iter().min();

    println!("\n\n{:?}", b);
}
