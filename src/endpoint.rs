use rori_utils::data::RoriData;
use rori_utils::client::{RoriClient, ConfigServer};
use rustc_serialize::json::decode;
use std::path::Path;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::io::prelude::*;
use std::process::Command;
use std::fs::File;

#[allow(dead_code)]
struct Client {
    stream: TcpStream,
}

#[allow(dead_code)]
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

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct RoriServer {
    rori_ip: Option<String>,
    rori_port: Option<String>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct EndpointDetails {
    owner: Option<String>,
    name: Option<String>,
    compatible_types: Option<String>,
}

#[allow(dead_code)]
pub struct Endpoint {
    address: String,
    rori_address: String,
    pub is_registered: bool,
    owner: String,
    name: String,
    compatible_types: String,
}

#[allow(dead_code)]
impl Endpoint {
    fn parse_config_server(data: String) -> String {
        let params: ConfigServer = decode(&data[..]).unwrap();
        format!("{}:{}",
                &params.ip.unwrap_or(String::from("")),
                &params.port.unwrap_or(String::from("")))
    }

    fn parse_config_rori(data: String) -> String {
        let params: RoriServer = decode(&data[..]).unwrap();
        format!("{}:{}",
                &params.rori_ip.unwrap_or(String::from("")),
                &params.rori_port.unwrap_or(String::from("")))
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
        let address = Endpoint::parse_config_server(data.clone());
        let rori_address = Endpoint::parse_config_rori(data.clone());
        let details: EndpointDetails = decode(&data[..]).unwrap();
        if address == ":" || rori_address == ":" {
            error!(target:"endpoint", "Empty config for the connection to the server");
        }
        Endpoint {
            address: address,
            rori_address: rori_address,
            is_registered: false,
            owner: details.owner.unwrap_or(String::from("")),
            name: details.name.unwrap_or(String::from("")),
            compatible_types: details.compatible_types.unwrap_or(String::from("")),
        }
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
                            .arg(&data_to_process.content)
                            .spawn()
                            .expect("ls command failed to start");
                    }
                    if data_to_process.datatype == "shell" {
                        info!(target:"endpoint", "Execute: {}", &data_to_process.content);
                        let output = Command::new("sh")
                            .arg("-c")
                            .arg(&*data_to_process.content)
                            .output()
                            .expect("failed to execute process");
                        let _ = output.stdout;
                    }
                }
                Err(e) => {
                    error!(target:"endpoint", "Connection failed because {}", e);
                }
            };
        }
        drop(listener);
    }

    pub fn register(&mut self) {
        info!(target:"endpoint", "try to register endpoint");
        // TODO security and if correctly registered
        let rori_address = self.rori_address.clone();
        let address = self.address.clone();
        let mut client = RoriClient { address: rori_address };
        let mut content = String::from(address);
        content.push_str("|");
        content.push_str(&*self.compatible_types);
        self.is_registered = client.send_to_rori(&self.owner, &*content, &self.name, "register");
    }
}
