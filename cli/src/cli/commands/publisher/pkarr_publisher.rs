use std::{thread, time::Duration, sync::mpsc::channel};
use chrono;
use pkarr::{Keypair, PkarrClient, SignedPacket};

pub struct PkarrPublisher {
    pub packet: SignedPacket,
    pub wait_time: Duration,
}

/**
 * Continuesly publishes the package
 */
impl PkarrPublisher {
    pub fn new(packet: SignedPacket) -> Self {
        PkarrPublisher {
            packet,
            wait_time: Duration::from_secs(60*60), // 1hr as recommended by the spec
        }
    }
    pub fn run_once(&self) -> Result<(), pkarr::Error> {
        let client = PkarrClient::new();
        let res = client.publish(&self.packet)?;
        Ok(())
    }

    pub fn run(&self){
        let (tx, rx) = channel();
    
        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
            .expect("Error setting Ctrl-C handler");
        println!("Stop with Ctrl-C...");
    
        loop {
            let result = self.run_once();
            if (result.is_ok()) {
                println!("{} Announced successfully.", chrono::offset::Local::now());
            } else {
                println!("{} Error {}", chrono::offset::Local::now(), result.unwrap_err().to_string());
            }

            let wait_result = rx.recv_timeout(self.wait_time);
            if wait_result.is_ok() {
                break;
            }
        }

        println!("Got it! Exiting...");
    }
}

#[cfg(test)]
mod tests {
    use pkarr::Keypair;

    

    use crate::cli::commands::publisher::csv_records::CsvRecords;

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
        let parsed = CsvRecords::from_csv(csv).unwrap();
        let packet = parsed.to_signed_packet(&keypair).unwrap();
        let publisher = PkarrPublisher::new(packet);
        publisher.run_once().unwrap();
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
        let parsed = CsvRecords::from_csv(csv).unwrap();
        let packet = parsed.to_signed_packet(&keypair).unwrap();
        let publisher = PkarrPublisher::new(packet);
        // publisher.run()
    }
}
