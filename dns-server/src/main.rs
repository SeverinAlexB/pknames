use std::{error::Error, sync::mpsc::channel};
use ctrlc;
use any_dns::{CustomHandler, Builder};
use pkarr::PkarrClient;
use pkarr_resolver::resolve_pkarr_pubkey;
mod pknames_resolver;
mod pkarr_resolver;

#[derive(Clone, Debug)]
struct MyHandler {
    client: PkarrClient
}

impl MyHandler {
    pub fn new() -> Self {
        Self {
            client: PkarrClient::new()
        }
    }
}

impl CustomHandler for MyHandler {
    fn lookup(&self, query: &Vec<u8>) -> std::prelude::v1::Result<Vec<u8>, Box<dyn Error>> {
        resolve_pkarr_pubkey(query, &self.client)
        // Err("not imp".into())
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");


    println!("Listening on 0.0.0.0:53. Waiting for Ctrl-C...");
    let anydns = Builder::new().handler(MyHandler::new()).threads(4).verbose(false).build();

    rx.recv().expect("Could not receive from channel.");

    println!("Got it! Exiting...");
    anydns.join();

    Ok(())
}
