use std::{convert::TryInto, net::Ipv4Addr};

use pkarr::{PkarrClient, PublicKey, dns::ResourceRecord, Error};
use pknames_core::resolve::resolve_standalone;
use crate::dns::protocol::{DnsRecord, TransientTtl};

use super::{protocol::{QueryType, DnsPacket, ResultCode}, resolve::ResolveError};

fn parse_pkarr_uri(qname: &str) -> Option<PublicKey> {
    if qname.len() == 55 && qname.starts_with("pk:") {
        let trying: Result<PublicKey,_> = qname.try_into();
        trying.ok()
    } else {
        None
    }
}


pub fn resolve_pknames(qname: &str, qtype: QueryType) -> Result<DnsPacket, ResolveError> {
    println!("Resolving pknames {}", qname);
    let result = resolve_standalone(qname, "~/.pknames");
    if let Err(e) = result {
        eprintln!("Error resolving pkname: {}", e);
        return Err(ResolveError::NoServerFound);
    };

    let prediction = result.unwrap();

    for class in prediction.classes.iter() {
        println!("{} {:.2}%", class.pubkey, class.probability * 100.0);
    };

    let best_class = prediction.get_best_class().expect("Class should be avail.");
    println!("Best class -> {} {:.2}%", best_class.pubkey, best_class.probability * 100.0);

    let pkarr = PkarrClient::new();

    let pubkey: PublicKey = best_class.pubkey.as_str().try_into().expect("Invalid zbase32 pkarr pubkey");
    let pkarr_result = pkarr.resolve(pubkey.clone());
    if pkarr_result.is_none() {
        eprintln!("Found nothing on mainline for the pubkey");
        return Err(ResolveError::NoServerFound);
    };
    let signed_packet = pkarr_result.unwrap();
    let records: Vec<&ResourceRecord> = signed_packet.resource_records(qname).collect();

    println!("Answers");
    for answer in records.iter() {
        println!("- {} {:?}", answer.name, answer.rdata);
    }

    let hermes_records = records.into_iter().map(|r| {
        let ip = match r.rdata.clone() {
            pkarr::dns::rdata::RData::A(addr) => {
                Ipv4Addr::from(addr.address)
            },
            _ => {
                let ip: Ipv4Addr = "127.1.1.99".parse().unwrap();
                ip
            }
        };

        DnsRecord::A { domain: qname.to_string(), addr: ip, ttl: TransientTtl(r.ttl) }
    }).collect();
    
    let mut packet = DnsPacket::new();
    packet.answers = hermes_records;

    packet.header.rescode = ResultCode::NOERROR;
    Ok(packet)
    
}




#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use pkarr::{SignedPacket, dns::{Packet, ResourceRecord, Name}, Keypair};

    use super::*;

    fn get_test_keypair() -> Keypair {
        let secret = "6kfe1u5jyqxg644eqfgk1cp4w9yjzwq51rn11ftysuo6xkpc64cy";
        let seed = zbase32::decode_full_bytes_str(secret).unwrap();
        let slice: &[u8; 32] = &seed[0..32].try_into().unwrap();
        let keypair = Keypair::from_secret_key(slice);
        keypair
    }

    #[test]
    fn publish_record() {
        let client = PkarrClient::new();
        let mut packet= Packet::new_reply(0);
        let ip: Ipv4Addr = "93.184.216.34".parse().unwrap();
        let record = ResourceRecord::new(Name::new("pknames.p2p").unwrap(), pkarr::dns::CLASS::IN, 100, pkarr::dns::rdata::RData::A(ip.try_into().unwrap()));
        packet.answers.push(record);

        let keypair = get_test_keypair();
        println!("Publish packet with pubkey {}", keypair.to_uri_string());
        let signed_packet = SignedPacket::from_packet(&keypair, &packet).unwrap();
        let result = client.publish(&signed_packet);
        result.expect("Should have published.");
    }
}
