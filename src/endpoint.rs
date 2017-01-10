use openssl::ssl::{SslContext, SslMethod, SslStream, SSL_VERIFY_NONE};
use openssl::x509::X509FileType::PEM;
use rori_utils::data::RoriData;
use rori_utils::endpoint::{Endpoint, Client, RoriEndpoint};
use std::path::Path;
use std::process::Command;
use std::net::TcpListener;

pub struct DesktopEndpoint {
    endpoint: RoriEndpoint,
}

#[allow(dead_code)]
impl Endpoint for DesktopEndpoint {
    fn start(&self) {
        let listener = TcpListener::bind(&*self.endpoint.address).unwrap();
        let mut ssl_context = SslContext::new(SslMethod::Tlsv1).unwrap();
        match ssl_context.set_certificate_file(&*self.endpoint.cert.clone(), PEM) {
            Ok(_) => info!(target:"Server", "Certificate set"),
            Err(_) => error!(target:"Server", "Can't set certificate file"),
        };
        ssl_context.set_verify(SSL_VERIFY_NONE, None);
        match ssl_context.set_private_key_file(&*self.endpoint.key.clone(), PEM) {
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
                        let data_authorized = self.is_authorized(data_to_process.clone());
                        if data_authorized {
                            // TODO security
                            if data_to_process.datatype == "music" {
                                Command::new("python3")
                                    .arg("scripts/music.py")
                                    .arg(&data_to_process.content)
                                    .spawn()
                                    .expect("music.py command failed to start");
                            }
                            if data_to_process.datatype == "alarm" {
                                Command::new("python3")
                                    .arg("scripts/alarm.py")
                                    .arg(&data_to_process.content)
                                    .spawn()
                                    .expect("alarm.py command failed to start");
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

    fn is_authorized(&self, data: RoriData) -> bool {
        self.endpoint.is_authorized(data)
    }

    fn register(&mut self) {
        self.endpoint.register()
    }
}

impl DesktopEndpoint {
    pub fn new<P: AsRef<Path>>(config: P) -> DesktopEndpoint {
        DesktopEndpoint { endpoint: RoriEndpoint::new(config) }
    }

    pub fn is_registered(&self) -> bool {
        self.endpoint.is_registered
    }
}
