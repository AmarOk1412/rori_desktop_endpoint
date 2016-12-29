use crypto::digest::Digest;
use crypto::sha2::Sha256;
use openssl::ssl::{SslContext, SslMethod, SslStream, SSL_VERIFY_NONE};
use openssl::x509::X509FileType::PEM;
use rori_utils::data::RoriData;
use rori_utils::client::{RoriClient, ConfigServer};
use rustc_serialize::json::decode;
use std::path::Path;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::io::prelude::*;
use std::process::Command;
use std::fs::File;

// TODO move in utils

#[allow(dead_code)]
struct Client {
    stream: SslStream<TcpStream>,
}

#[allow(dead_code)]
impl Client {
    fn new(stream: SslStream<TcpStream>) -> Client {
        return Client { stream: stream };
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
struct AuthorizedUser {
    pub name: Option<String>,
    pub secret: Option<String>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct RoriServer {
    rori_ip: Option<String>,
    rori_port: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub secret: Option<String>,
    pub authorize: Vec<AuthorizedUser>,
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
    cert: String,
    key: String,
    secret: String,
    authorize: Vec<AuthorizedUser>,
}

#[allow(dead_code)]
impl Endpoint {
    fn parse_config_server(data: String) -> String {
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
        let address = Endpoint::parse_config_server(data.clone());
        let params: RoriServer = decode(&data[..]).unwrap();
        let rori_address = format!("{}:{}",
                                   &params.rori_ip.unwrap_or(String::from("")),
                                   &params.rori_port.unwrap_or(String::from("")));
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
            cert: params.cert.unwrap_or(String::from("")),
            key: params.key.unwrap_or(String::from("")),
            secret: params.secret.unwrap_or(String::from("")),
            authorize: params.authorize,
        }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(&*self.address).unwrap();
        let mut ssl_context = SslContext::new(SslMethod::Tlsv1).unwrap();
        match ssl_context.set_certificate_file(&*self.cert.clone(), PEM) {
            Ok(_) => info!(target:"Server", "Certificate set"),
            Err(_) => error!(target:"Server", "Can't set certificate file"),
        };
        ssl_context.set_verify(SSL_VERIFY_NONE, None);
        match ssl_context.set_private_key_file(&*self.key.clone(), PEM) {
            Ok(_) => info!(target:"Server", "Private key set"),
            Err(_) => error!(target:"Server", "Can't set private key"),
        };
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {

                    let ssl_stream = SslStream::accept(&ssl_context, stream.try_clone().unwrap());
                    let ssl_ok = match ssl_stream {
                        Ok(_) => true,
                        Err(_) => false,
                    };
                    if ssl_ok {
                        let ssl_stream = ssl_stream.unwrap();
                        let mut client = Client::new(ssl_stream.try_clone().unwrap());
                        let data_received = client.read();
                        let end = data_received.find(0u8 as char);
                        let (data_received, _) =
                            data_received.split_at(end.unwrap_or(data_received.len()));
                        let data_to_process = RoriData::from_json(String::from(data_received));
                        let data_authorized = Endpoint::is_authorized(self.authorize.clone(),
                                                                      data_to_process.clone());
                        if data_authorized {
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
                        } else {
                            error!(target:"Server", "Can't create SslStream");
                        }
                    } else {
                        error!(target:"Server", "Can't create SslStream");
                    }
                }
                Err(e) => {
                    error!(target:"endpoint", "Connection failed because {}", e);
                }
            };
        }
        drop(listener);
    }

    fn is_authorized(authorize: Vec<AuthorizedUser>, data: RoriData) -> bool {
        let mut hasher = Sha256::new();
        hasher.input_str(&*data.secret);
        let secret = hasher.result_str();
        for client in authorize {
            if client.name.unwrap().to_lowercase() == data.client.to_lowercase() &&
               secret.to_lowercase() == client.secret.unwrap().to_lowercase() {
                return true;
            }
        }
        false
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
        self.is_registered =
            client.send_to_rori(&self.owner, &*content, &self.name, "register", &self.secret);
    }
}
