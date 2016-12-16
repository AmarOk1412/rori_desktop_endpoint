extern crate rustc_serialize;
extern crate regex;

mod rori_utils;

use std::io::prelude::*;
use std::path::Path;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use rori_utils::client::ConfigServer;
use rori_utils::data::RoriData;
use rustc_serialize::json::decode;
use std::fs::File;
use std::process::Command;

struct Client {
    stream: TcpStream,
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        Client { stream: stream }
    }

    fn read(&mut self) -> String {
        let mut result = String::from("");
        let mut buffer = [0u8; 512];
        loop {
            let usize = self.stream.read(&mut buffer).unwrap();
            if usize == 0 {
                break;
            }
            let msg = from_utf8(&buffer).unwrap();
            result.push_str(msg);
        }
        result
    }
}

pub struct Endpoint {
    address: String,
}

impl Endpoint {
    pub fn parse_config(data: String) -> String {
        let params: ConfigServer = decode(&data[..]).unwrap();

        format!("{}:{}",
                &params.ip.unwrap_or(String::from("")),
                &params.port.unwrap_or(String::from("")))
    }

    pub fn new<P: AsRef<Path>>(config: P) -> Endpoint {
        // Configure from file
        let mut file = File::open(config)
            .ok()
            .expect("Config file not found");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .ok()
            .expect("failed to read!");
        let address = Endpoint::parse_config(data);
        if address == ":" {
            println!("Empty config for the connection to the server");
        }
        Endpoint { address: address }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(&*self.address).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut client = Client::new(stream.try_clone().unwrap());
                    let data_received = client.read();
                    let end = data_received.find(0u8 as char);
                    let (data_received, _) =
                        data_received.split_at(end.unwrap_or(data_received.len()));
                    let data_to_process = RoriData::from_json(String::from(data_received));
                    // TODO security
                    if data_to_process.datatype == "music" {
                        Command::new("python3")
                            .arg("scripts/music.py")
                            .arg(data_to_process.content)
                            .spawn()
                            .expect("ls command failed to start");
                    }
                }
                Err(e) => {
                    println!("Connection failed because {}", e);
                }
            };
        }
        drop(listener);
    }
}

// Launch RoriIrcEntry
fn main() {
    let endpoint = Endpoint::new("config_server.json");
    endpoint.start();
}
