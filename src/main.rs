extern crate env_logger;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate regex;
extern crate rustc_serialize;

mod rori_utils;
mod endpoint;

use endpoint::Endpoint;

fn main() {
    // Init logging
    env_logger::init().unwrap();

    let mut endpoint = Endpoint::new("config_server.json");
    endpoint.register();
    if endpoint.is_registered {
        endpoint.start();
    } else {
        error!("Endpoint is not registered.");
    }
}
