#![allow(dead_code, unused_variables, unused_imports, unused_must_use,)]

#[macro_use]
extern crate clap;
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

use std::fs::File;
use std::path::Path;
use std::fs::OpenOptions;
use std::collections::HashMap;

use crossroad_server::crossroad::*;
use crossroad_server::traffic_protocol::*;
use crossroad_server::traffic_controls::*;
use crossroad_server::default_crossroad;
use crossroad_server::error::{Result, Error, JsonError};


fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Rutger S.")
        (about: "Awesome crossroad simulator!")
        (@arg ip: +required "Runs the server on this ip")
        (@arg port: -p --port +takes_value "Sets the port")
        (@arg json: -j --json +takes_value "Determines how the json output is encoded. Takes none, null or empty as the value.
            none:  Sends only the {banan} json vec.
            null:  Sends the complete {banen, busbanen, stoplichten} json, where the empty ones will be null.
            empty: Sends the complete {banen, busbanen, stoplichten} json, where the empty ones will be []\n")
    ).get_matches();

    let j_str = matches.value_of("json").unwrap_or("none");
    match JsonCompatLevel::from_str(&j_str) {
        Some(compat_level) => unsafe {
            crossroad_server::traffic_protocol::JSON_COMPAT_LEVEL = compat_level;
        },
        None => println!("Incorrect -j value!"),
    }

    let ip = matches.value_of("ip").unwrap();
    let port = matches.value_of("port").unwrap_or("9990");
    let address = format!("{}:{}", ip, port);

    println!("\nJson compatibility level = {:?} ", j_str);
    run_server(&*address).unwrap();
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

    let (log_file_recv, log_file_sent) = create_log_files(&client_stream).expect("log files");

    // Convert stream to buffered streams
    let client_reader = BufReader::new(try!(client_stream.try_clone()));
    let client_writer = BufWriter::new(client_stream);

    // Main thread uses this channel to send (json) updates to the client.
    let (out_transmitter, out_receiver) = channel::<String>();
    let (exit_main_loop_tx, exit_main_loop_rx) = channel();

    // Getting updates from the simulator(client) via a socket, so make it safe with reference counter + a mutex.
    let client_baan_sensor_states = Arc::new(Mutex::new(SensorStates::new()));

    // Run seperate threads
    let client_receiver_handle = spawn_client_sensor_receiver(client_reader, client_baan_sensor_states.clone(), log_file_recv);
    let client_updater_handle = spawn_client_updater(client_writer, out_receiver, log_file_sent);
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

fn create_log_files(client_stream: &TcpStream) -> io::Result<(File, File)> {
    let ip = try!(client_stream.local_addr().map(|sock| sock.ip()));
    let file_name = format!("{}_{}",  time::now().strftime("%e-%m-%G_%k%M_").unwrap(), ip);
    let path_in = format!("{}_{}.log", file_name, "received");
    let path_out = format!("{}_{}.log", file_name, "sent");

    let mut o = OpenOptions::new();
    let u = o.create(true).append(true);

    let log_file_recv = try!(u.open(Path::new(&path_in)));
    let log_file_sent = try!(u.open(Path::new(&path_out)));

    Ok((log_file_recv, log_file_sent))
}

fn spawn_main_loop( out_tx: Sender<String>,
                    exit_rx: Receiver<u8>,
                    sensor_shared_state: Arc<Mutex<SensorStates>>)
                    -> JoinHandle<Result<()>>
 {
    thread::spawn(move || {

        let traffic_lights = default_crossroad::create_traffic_lights();
        let traffic_controls = default_crossroad::create_traffic_controls(&traffic_lights);
        let crossroad = default_crossroad::create_crossroad(&traffic_controls);

        let mut crossroad_state = CrossroadState::AllRed;
        let mut time = 0; // seconds

        let frequency_scheduler = sched::periodic_ms(1000);

        if true { // TESTS
            // crossroad.send_all_bulk(&out_tx, JsonState::Groen);
            // crossroad.send_all(&out_tx, JsonState::Geel);
            // crossroad.send_all_bulk(&out_tx, JsonState::Rood);
        }

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

fn spawn_client_sensor_receiver(mut reader: BufReader<TcpStream>, sensor_data: Arc<Mutex<SensorStates>>, mut log_file: File) -> JoinHandle<Result<()>> {

    thread::spawn(move || {
        loop {
            let mut line = String::new();

            try!(reader.read_line(&mut line));
            let ref mut traffic_state = *sensor_data.lock().unwrap();

            log_file.write(format!("\n{}\n", time::now().strftime("%T").unwrap()).as_bytes());
            log_file.write_all(&line.as_bytes());

            match serde_json::from_str::<ProtocolJson>(&line) {
                Ok(protocol_obj) => {

                    if let Some(ref banen) = protocol_obj.banen {

                        if banen.len() > 0 {
                            traffic_state.update(banen);
                            //println!("Client->Server: received baan sensor update: {:?} new_state = {:?}", banen, traffic_state)
                        }
                    }

                    if let Some(ref busbanen) = protocol_obj.busbanen {

                        if busbanen.len() > 0 {
                            traffic_state.update_bussen(busbanen);
                            println!("Client->Server: received BUSBAAN sensor update: {:?} new_state = {:?}", busbanen, traffic_state)
                        }
                    }
                },
                Err(err) => println!("Client->Server: received faulty json string {:?}", line),
            }
        }
    })
}

fn spawn_client_updater(mut writer: BufWriter<TcpStream>, rx: Receiver<String>, mut log_file: File) -> thread::JoinHandle<Result<()>> {
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(msg) => {
                    log_file.write(format!("\n\n{}\n", time::now().strftime("%T").unwrap()).as_bytes());
                    log_file.write_all(&msg.as_bytes());

                    try!(writer.write(format!("{}\r\n", &msg).as_bytes()));
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
