#![allow(dead_code, unused_variables, unused_imports)]

extern crate serde;
extern crate serde_json;
extern crate schedule_recv;
extern crate crossroad_server; // Local crate


use serde::ser;
use schedule_recv as sched;

use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::fmt::Display;
use std::io::{self, BufRead, Write, BufReader, BufWriter};
use std::thread;
use std::thread::{JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

use crossroad_server::traffic_protocol::*;
use crossroad_server::error::{Result, Error, JsonError};


fn main() {
    run_server("127.0.0.1:80").unwrap();    
}

fn run_server<A>(address: A) -> io::Result<()> where A: ToSocketAddrs + Display {

    let listener = try!(TcpListener::bind(&address));
    println!("Server listening on: {}", address);

    // Infinite loop.
    for tcp_stream in listener.incoming().filter_map(|i| i.ok()) {     
        thread::spawn(move || {
            println!("Connecting a new client");
            
            match handle_client(tcp_stream) {
                Ok(_) => println!("Client disconnected"),
                Err(v) => println!("Client Handle Error {:?}", v),
            };
        });
    }

    Ok(())
}

fn handle_client(client_stream: TcpStream) -> io::Result<()> {

    // Convert stream to buffered streams
    let client_writer = BufWriter::new(try!(client_stream.try_clone()));
    let client_reader = BufReader::new(client_stream); 

    // Main thread uses this channel to send (json) updates to the client.
    let (out_transmitter, out_receiver) = channel::<String>();

    // Getting updates from the simulator(client) via a socket, so make it safe with reference counter + a mutex.
    let client_baan_sensor_states = Arc::new(Mutex::new(BaanSensorStates::new()));

    // Run seperate threads
    let client_receiver_handle = spawn_client_sensor_receiver(client_reader, client_baan_sensor_states.clone());
    let client_updater_handle = spawn_client_updater(client_writer, out_receiver);
    let verkeersregelinstallatie_handle = spawn_main_loop(out_transmitter, client_baan_sensor_states.clone());

    println!("Connection established");

    // Join all threads together
    let joins = client_updater_handle.join()
        .and_then(|_| client_receiver_handle.join())
        .and_then(|_| verkeersregelinstallatie_handle.join());

    Ok(())
}

fn spawn_main_loop(tx: Sender<String>, sensor_shared_state: Arc<Mutex<BaanSensorStates>>) -> JoinHandle<Result<()>> {
    thread::spawn(move || {
        let frequency_scheduler = sched::periodic_ms(1000);
        loop {       
            frequency_scheduler.recv().unwrap();

            let ref mut sensor_state = *sensor_shared_state.lock().unwrap();

            // TODO: Traffic logic
            // TODO: send client new state

            tx.send("New stoplicht state".to_string());
        }
    })
}

fn spawn_client_sensor_receiver(mut reader: BufReader<TcpStream>, sensor_data: Arc<Mutex<BaanSensorStates>>) -> JoinHandle<Result<()>> {  
    thread::spawn(move || {
        let mut line = String::new();
        loop {  
            try!(reader.read_line(&mut line));
            //let ref mut traffic_state = *sensor_data.lock().unwrap();            
            //let baan = try!(traffic_state.update_from_json(&line)); 
            println!("Client->Server: received baan sensor update: {:?}", line);     
        }
    })
}

fn spawn_client_updater(mut writer: BufWriter<TcpStream>, rx: Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let msg = rx.recv().unwrap();
            writer.write(&msg.as_bytes()).unwrap();
            writer.flush().unwrap();
            println!("Server->Client: sent new stoplicht state {:?}", msg);
        }
    })   
}

#[test]
fn verkeer() {
/*

        let west = XorStoplichtGroep {
            naam: "Richting-West".to_string(),
        };
*/
}