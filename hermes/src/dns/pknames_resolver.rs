use std::{convert::TryInto, net::Ipv4Addr};

use pkarr::{PkarrClient, PublicKey, dns::ResourceRecord};
use pknames_core::resolve::resolve_standalone;
use crate::dns::protocol::{DnsRecord, TransientTtl};

use super::{protocol::{QueryType, DnsPacket, ResultCode}, resolve::ResolveError};

fn parse_pkarr_uri(uri: &str) -> Option<PublicKey> {
    if !uri.ends_with(".home") {
        return None
    }
    let uri = uri.to_string().replace(".home", "");
    let trying: Result<PublicKey,_> = uri.as_str().try_into();
    trying.ok()
}

pub fn resolve_pkarr_pubkey(qname: &str, pubkey: PublicKey, qtype: QueryType) -> Option<DnsPacket> {
    let pkarr = PkarrClient::new();
    let signed_packet = pkarr.resolve_most_recent(pubkey.clone())?;
    println!("Answers");
    for answer in signed_packet.packet().answers.iter() {
        println!("- {} {:?}", answer.name, answer.rdata);
    }
    let records: Vec<&ResourceRecord> = signed_packet.resource_records(qname).collect();



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
    Some(packet)
}


pub fn resolve_pknames(qname: &str, qtype: QueryType) -> Result<DnsPacket, ResolveError> {
    println!("Resolving pknames {}", qname);

    let pubkey = parse_pkarr_uri(qname);
    if pubkey.is_some() {
        let pubkey = pubkey.unwrap();
        let result = resolve_pkarr_pubkey(&pubkey.to_z32(), pubkey, qtype);
        if result.is_none() {
            return Err(ResolveError::NoServerFound);
        } else {
            println!("Domain is pkarr uri");
            return Ok(result.unwrap())
        }
    }

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

    let pubkey = parse_pkarr_uri(&best_class.pubkey).expect("Should be pkarr pubkey");
    let result = resolve_pkarr_pubkey(qname, pubkey, qtype);
    if result.is_none() {
        return Err(ResolveError::NoServerFound);
    } else {
        return Ok(result.unwrap())
    } 
}




#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use pkarr::{SignedPacket, dns::{Packet, ResourceRecord, Name}, Keypair};

    use super::*;

    fn get_test_keypair() -> Keypair {
        let secret = "6kfe1u5jyqxg644eqfgk1cp4w9yjzwq51rn11ftysuo6xkpc64by";
        let seed = zbase32::decode_full_bytes_str(secret).unwrap();
        let slice: &[u8; 32] = &seed[0..32].try_into().unwrap();
        let keypair = Keypair::from_secret_key(slice);
        keypair
    }

    #[test]
    fn publish_record() {
        let keypair = get_test_keypair();
        let uri = keypair.to_uri_string();
        let plain_pubkey = keypair.to_z32();
        println!("Publish packet with pubkey {}", uri);
        let client = PkarrClient::new();
        let mut packet= Packet::new_reply(0);
        let ip: Ipv4Addr = "93.184.216.34".parse().unwrap();
        let record = ResourceRecord::new(Name::new("pknames.p2p").unwrap(), pkarr::dns::CLASS::IN, 100, pkarr::dns::rdata::RData::A(ip.try_into().unwrap()));
        packet.answers.push(record);
        let record = ResourceRecord::new(Name::new(".").unwrap(), pkarr::dns::CLASS::IN, 100, pkarr::dns::rdata::RData::A(ip.try_into().unwrap()));
        packet.answers.push(record);


        let signed_packet = SignedPacket::from_packet(&keypair, &packet).unwrap();
        let result = client.publish(&signed_packet);
        result.expect("Should have published.");
    }

    #[test]
    fn parsed_pkarr() {
        let keypair = get_test_keypair();
        let qname = format!("{}.home", keypair.to_uri_string());
        let result = resolve_pkarr_pubkey(&keypair.to_z32(), keypair.public_key(), QueryType::A);
        let packet = result.unwrap();
        for answer in packet.answers.iter() {
            println!("{}, {:?}", answer.get_domain().unwrap(), answer.get_querytype());
        }

    }
}
