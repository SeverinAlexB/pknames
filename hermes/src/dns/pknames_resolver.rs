use std::{
    convert::{TryFrom, TryInto},
    net::{Ipv4Addr, Ipv6Addr}, any::Any,
};

use super::{
    protocol::{DnsPacket, QueryType, ResultCode},
    resolve::ResolveError,
};
use crate::dns::protocol::{DnsRecord, TransientTtl};
use pkarr::dns::rdata::RData;
use pkarr::{
    dns::{rdata::SOA, ResourceRecord},
    PkarrClient, PublicKey,
};
use pknames_core::resolve::resolve_standalone;

// macros::rdata_enum! {
//     A, ✅
//     AAAA, ✅
//     NS<'a>, ✅
//     MD<'a>,
//     CNAME<'a>, ✅
//     MB<'a>,
//     MG<'a>,
//     MR<'a>,
//     PTR<'a>,
//     MF<'a>,
//     HINFO<'a>,
//     MINFO<'a>,
//     MX<'a>, ✅
//     TXT<'a>, ✅
//     SOA<'a>, ✅
//     WKS<'a>,
//     SRV<'a>, ✅
//     RP<'a>,
//     AFSDB<'a>,
//     ISDN<'a>,
//     RouteThrough<'a>,
//     NSAP,
//     NSAP_PTR<'a>,
//     LOC,
//     OPT<'a>,
//     CAA<'a>,
// }

impl<'a> TryFrom<ResourceRecord<'a>> for DnsRecord {
    type Error = String;
    fn try_from(value: ResourceRecord<'a>) -> Result<Self, Self::Error> {
        let domain = value.name.to_string();
        let ttl = TransientTtl(value.ttl);
        match value.rdata {
            RData::A(addr) => {
                let ip = Ipv4Addr::from(addr.address);
                Ok(DnsRecord::A {
                    domain,
                    ttl,
                    addr: ip,
                })
            }
            RData::CNAME(val) => {
                let host = val.0.to_string();
                Ok(DnsRecord::CNAME { domain, ttl, host })
            }
            RData::TXT(val) => {
                let data = val.try_into().expect("Should be valid cname string");
                Ok(DnsRecord::TXT { domain, ttl, data })
            }
            RData::AAAA(addr) => {
                let ip = Ipv6Addr::from(addr.address);
                Ok(DnsRecord::AAAA {
                    domain,
                    ttl,
                    addr: ip,
                })
            }
            RData::NS(ns) => {
                let host = ns.to_string();
                Ok(DnsRecord::NS { domain, host, ttl })
            }
            RData::SOA(soa) => Ok(DnsRecord::SOA {
                domain,
                m_name: soa.mname.to_string(),
                r_name: soa.rname.to_string(),
                serial: soa.serial,
                refresh: soa.refresh as u32,
                retry: soa.retry as u32,
                expire: soa.expire as u32,
                minimum: soa.minimum,
                ttl,
            }),
            RData::SRV(srv) => Ok(DnsRecord::SRV {
                domain,
                priority: srv.priority,
                weight: srv.weight,
                port: srv.port,
                host: srv.target.to_string(),
                ttl,
            }),
            RData::MX(mx) => Ok(DnsRecord::MX {
                domain,
                ttl,
                priority: mx.preference,
                host: mx.exchange.to_string(),
            }),
            _ => Err("DNS record type is not implemented.".to_string()),
        }
    }
}

fn parse_pkarr_uri(uri: &str) -> Option<PublicKey> {
    let trying: Result<PublicKey, _> = uri.try_into();
    trying.ok()
}

/**
 * Resolves a domain with pkarr.
 */
pub fn resolve_pkarr_pubkey(qname: &str, pubkey: PublicKey, qtype: QueryType) -> Option<DnsPacket> {
    let pkarr = PkarrClient::new();
    let signed_packet = pkarr.resolve(pubkey.clone())?;
    let records: Vec<&ResourceRecord> = signed_packet.resource_records(qname).collect();
    let hermes_records = records
        .into_iter()
        .filter_map(|r| {
            let record = r.clone();
            let hermes_record: Result<DnsRecord, _> = record.try_into();
            if hermes_record.is_err() {
                println!(
                    "Warn: Failed to convert pkarr record - {}",
                    hermes_record.clone().unwrap_err()
                )
            };
            hermes_record.ok()
        }).filter(|record| {
            record.get_querytype() == qtype
        })
        .collect();

    let mut packet = DnsPacket::new();
    packet.answers = hermes_records;

    packet.header.rescode = ResultCode::NOERROR;
    Some(packet)
}

/**
 * Resolves pknames and pkarr public keys.
 */
pub fn resolve_pknames_or_pkarr_pubkey(qname: &str, qtype: QueryType) -> Result<DnsPacket, ResolveError> {
    let pubkey = parse_pkarr_uri(qname);
    if pubkey.is_some() {
        let pubkey = pubkey.unwrap();
        let result = resolve_pkarr_pubkey(&pubkey.to_z32(), pubkey, qtype);
        if result.is_none() {
            return Err(ResolveError::NoServerFound);
        } else {
            return Ok(result.unwrap());
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
    }

    let best_class = prediction.get_best_class().expect("Class should be available.");
    let pubkey = parse_pkarr_uri(&best_class.pubkey).expect("Should be pkarr pubkey");
    let result = resolve_pkarr_pubkey(qname, pubkey, qtype);
    if result.is_none() {
        return Err(ResolveError::NoServerFound);
    } else {
        return Ok(result.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use pkarr::{
        dns::{Name, Packet, ResourceRecord},
        Keypair, SignedPacket,
    };
    use std::net::Ipv4Addr;

    use super::*;

    fn get_test_keypair() -> Keypair {
        // pk:cb7xxx6wtqr5d6yqudkt47drqswxk57dzy3h7qj3udym5puy9cso
        let secret = "6kfe1u5jyqxg644eqfgk1cp4w9yjzwq51rn11ftysuo6xkpc64by";
        let seed = zbase32::decode_full_bytes_str(secret).unwrap();
        let slice: &[u8; 32] = &seed[0..32].try_into().unwrap();
        let keypair = Keypair::from_secret_key(slice);
        keypair
    }

    fn publish_record() {
        let keypair = get_test_keypair();
        // let uri = keypair.to_uri_string();
        // println!("Publish packet with pubkey {}", uri);

        let mut packet = Packet::new_reply(0);
        let ip: Ipv4Addr = "93.184.216.34".parse().unwrap();
        let record = ResourceRecord::new(
            Name::new("pknames.p2p").unwrap(),
            pkarr::dns::CLASS::IN,
            100,
            pkarr::dns::rdata::RData::A(ip.try_into().unwrap()),
        );
        packet.answers.push(record);
        let record = ResourceRecord::new(
            Name::new(".").unwrap(),
            pkarr::dns::CLASS::IN,
            100,
            pkarr::dns::rdata::RData::A(ip.try_into().unwrap()),
        );
        packet.answers.push(record);
        let signed_packet = SignedPacket::from_packet(&keypair, &packet).unwrap();
        
        let client = PkarrClient::new();
        let result = client.publish(&signed_packet);
        result.expect("Should have published.");
    }

    #[test]
    fn query_pubkey() {
        publish_record();

        let keypair = get_test_keypair();
        let result = resolve_pkarr_pubkey(&keypair.to_z32(), keypair.public_key(), QueryType::A);
        let packet = result.unwrap();
        assert_eq!(packet.answers.len(), 1);
        let answer = packet.answers.get(0).unwrap();
        assert_eq!(answer.get_querytype(), QueryType::A);
        assert_eq!(answer.get_ttl(), 100);
        assert_eq!(answer.get_domain().unwrap(), "cb7xxx6wtqr5d6yqudkt47drqswxk57dzy3h7qj3udym5puy9cso");
    }

    #[test]
    fn query_domain() {
        publish_record();

        let keypair = get_test_keypair();
        let result = resolve_pkarr_pubkey("pknames.p2p", keypair.public_key(), QueryType::A);
        let packet = result.unwrap();
        assert_eq!(packet.answers.len(), 1);
        let answer = packet.answers.get(0).unwrap();
        assert_eq!(answer.get_querytype(), QueryType::A);
        assert_eq!(answer.get_ttl(), 100);
        assert_eq!(answer.get_domain().unwrap(), "pknames.p2p.cb7xxx6wtqr5d6yqudkt47drqswxk57dzy3h7qj3udym5puy9cso");
    }
}
