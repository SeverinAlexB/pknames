use std::{sync::mpsc::channel, io::{self, Write}};
use chrono;
use pkarr::{PkarrClient, SignedPacket};

pub struct PkarrPublisher {
    pub packet: SignedPacket
}

/**
 * Continuously publishes dns records to pkarr
 */
impl PkarrPublisher {
    pub fn new(packet: SignedPacket) -> Self {
        PkarrPublisher {
            packet
        }
    }
    
    pub fn run_once(&self) -> () {
        let client = PkarrClient::new();
        print!("Hang on...");
        io::stdout().flush().unwrap();
        let result = client.publish(&self.packet);
        print!("\r");
        // std::io::stdout().flush();
        if result.is_ok() {
            println!("{} Successfully announced.", chrono::offset::Local::now());
        } else {
            println!("{} Error {}", chrono::offset::Local::now(), result.unwrap_err().to_string());
        };

    }

    pub fn run(&self, interval: chrono::Duration){
        let (tx, rx) = channel();
    
        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
            .expect("Error setting Ctrl-C handler");
        loop {
            self.run_once();

            let wait_result = rx.recv_timeout(interval.to_std().expect("Valid duration expected"));
            if wait_result.is_ok() {
                break;
            }
        }
        println!();
        println!("Got it! Exiting...");
    }
}

#[cfg(test)]
mod tests {
    use pkarr::Keypair;
    use simple_dns::Packet;
    use crate::commands::pkarr::pkarr_records::PkarrRecords;
    use super::PkarrPublisher;

    fn get_test_keypair() -> Keypair {
        // pk:cb7xxx6wtqr5d6yqudkt47drqswxk57dzy3h7qj3udym5puy9cso
        let secret = "6kfe1u5jyqxg644eqfgk1cp4w9yjzwq51rn11ftysuo6xkpc64be";
        let seed = zbase32::decode_full_bytes_str(secret).unwrap();
        let slice: &[u8; 32] = &seed[0..32].try_into().unwrap();
        let keypair = Keypair::from_secret_key(slice);
        keypair
    }

    #[test]
    fn run_once() {
        let csv = "
        # Type, Domain, Data, TTL
        A, pknames.p2p, 127.0.0.1, 10
        TXT, test, helloworld,
        TXT, test, helloworld, a
        ";

        let keypair = get_test_keypair();
        let parsed = PkarrRecords::from_conf(csv).unwrap();
        let packet = parsed.to_signed_packet(&keypair).unwrap();
        let publisher = PkarrPublisher::new(packet);
        publisher.run_once();
    }

    #[test]
    fn run() {
        let csv = "
        # Type, Domain, Data, TTL
        A, pknames.p2p, 127.0.0.1, 10
        TXT, test, helloworld,
        TXT, test, helloworld, a
        ";

        let keypair = get_test_keypair();
        let parsed = PkarrRecords::from_conf(csv).unwrap();
        let packet = parsed.to_signed_packet(&keypair).unwrap();
        let publisher = PkarrPublisher::new(packet);
        // publisher.run(Duration::from_secs(60*60))
    }
}
