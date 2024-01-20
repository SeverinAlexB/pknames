use std::{path::Path, fs, error::Error, net::{Ipv4Addr, Ipv6Addr}, fmt::Display};

use csv::Trim;
use pkarr::{dns::{ResourceRecord, Name, rdata::{RData, CNAME, TXT}, Packet}, SignedPacket, Keypair};

pub struct PkarrRecord {
    pub typ: String,
    pub domain: String,
    pub data: String,
    pub ttl: u32
}


pub const DEFAULT_TTL: u32 = 43200; // 12 hours


impl PkarrRecord {
    fn to_resource_record<'a>(&'a self) -> Result<ResourceRecord<'a>, Box<dyn Error>> {
        let name: Name<'a> = Name::try_from(self.domain.as_str())?;
        let rdata = match self.typ.as_str() {
            "A" => {
                let ip: Ipv4Addr = self.data.parse()?;
                RData::A(ip.into())
            },
            "AAAA" => {
                let ip: Ipv6Addr = self.data.parse()?;
                RData::AAAA(ip.into())
            },
            "CNAME" => {
                let name: Name<'a> = Name::try_from(self.data.as_str())?;
                RData::CNAME(CNAME(name))
            },
            "TXT" => {
                let mut txt = TXT::new();
                txt.add_string(&self.data)?;
                RData::TXT(txt)
            },
            _ => {
                todo!("To be implemented record type")
            }
        };
        Ok(ResourceRecord::new(name, pkarr::dns::CLASS::IN, self.ttl, rdata))
    }
}

impl Display for PkarrRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {} {}", self.typ, self.domain, self.data, self.ttl)
    }
}

pub struct PkarrRecords {
    pub records: Vec<PkarrRecord>
}

impl PkarrRecords {
    pub fn from_path(csv_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(csv_path)?;
        Self::from_tabfile(&content)
    }

    pub fn from_tabfile(content: &str) -> Result<Self, Box<dyn Error>> {
        let mut list: Vec<PkarrRecord> = vec![];
        let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b' ')
        .has_headers(false)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(content.as_bytes());
        for result in rdr.records() {
            if result.is_err() {
                break;
            };
            let record = result?;
            let is_comment = record.as_slice().contains("#");
            if is_comment {
                continue;
            };
            let values: Vec<&str> = record.into_iter().filter(|val| !val.is_empty()).collect();
            if values.len() < 3 {
                continue;
            }
            let typ: String = values.get(0).unwrap().parse()?;
            let domain: String = values.get(1).unwrap().parse()?;
            let data: String = values.get(2).unwrap().parse()?;

            let mut ttl = DEFAULT_TTL;
            if values.len() >= 4 {
                let ttl_res: Result<u32, _> = values.get(3).unwrap().parse();
                let ttl_option = ttl_res.ok();
                ttl = ttl_option.unwrap_or(DEFAULT_TTL);
            }


            list.push(PkarrRecord{
                typ: typ.to_uppercase(),
                domain,
                data, 
                ttl
            })

        };
        Ok(PkarrRecords {
            records: list
        })
    }

    pub fn to_signed_packet(&self, keypair: &Keypair) -> Result<SignedPacket, Box<dyn Error>> {
        // Check for record validity
        for record in self.records.iter() {
            record.to_resource_record()?;
        };

        let mut packet = Packet::new_reply(0);
        packet.answers = self.records.iter().map(|r| {
            r.to_resource_record().unwrap()
        }).collect();

        Ok(SignedPacket::from_packet(&keypair, &packet)?)
    }
}

impl TryFrom<ResourceRecord<'_>> for PkarrRecord {
    type Error = String;

    fn try_from(value: ResourceRecord<'_>) -> Result<Self, Self::Error> {
        let domain = value.name.to_string();
        let ttl = value.ttl;

        match value.rdata {
            RData::A(a) => {
                Ok(PkarrRecord {
                    domain,
                    ttl,
                    typ: "A".to_string(),
                    data: Ipv4Addr::from(a.address).to_string()
                })
            },
            RData::AAAA(a) => {
                Ok(PkarrRecord {
                    domain,
                    ttl,
                    typ: "AAAA".to_string(),
                    data: Ipv6Addr::from(a.address).to_string()
                })
            },
            RData::TXT(a) => {
                Ok(PkarrRecord {
                    domain,
                    ttl,
                    typ: "TXT".to_string(),
                    data: a.try_into().expect("Should be valid txt string")
                })
            },
            RData::CNAME(val) => {
                let host = val.0.to_string();
                Ok(PkarrRecord {
                    domain,
                    ttl,
                    typ: "CNAME".to_string(),
                    data: host
                })
            }
            _ => {
                Err("Not implemented record typ".to_string())
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use pkarr::Keypair;

    use crate::commands::publisher::pkarr_records::{PkarrRecords, DEFAULT_TTL};

    fn get_test_keypair() -> Keypair {
        // pk:cb7xxx6wtqr5d6yqudkt47drqswxk57dzy3h7qj3udym5puy9cso
        let secret = "6kfe1u5jyqxg644eqfgk1cp4w9yjzwq51rn11ftysuo6xkpc64by";
        let seed = zbase32::decode_full_bytes_str(secret).unwrap();
        let slice: &[u8; 32] = &seed[0..32].try_into().unwrap();
        let keypair = Keypair::from_secret_key(slice);
        keypair
    }



    #[test]
    fn to_signed_packet() {
        let csv = "
        # Type, Domain, Data, TTL
        A pknames.p2p 127.0.0.1 10
        TXT  test helloworld
        ";
        let keypair = get_test_keypair();
        let parsed = PkarrRecords::from_tabfile(csv).unwrap();
        let signed_packet = parsed.to_signed_packet(&keypair).expect("Valid csv");
        let packet = signed_packet.packet();
        assert_eq!(packet.answers.len(), 2)
    }

    #[test]
    fn parse_tabfile() {
        let csv = "
        # Type Domain   Data TTL
         A pknames.p2p   \"127.0.0.1 yolo\" 10
        TXT  test  helloworld
        ";
        let parsed = PkarrRecords::from_tabfile(csv).unwrap();
        assert_eq!(parsed.records.len(), 2);

        let ele = parsed.records.get(0).unwrap();
        assert_eq!(ele.typ, "A");
        assert_eq!(ele.domain, "pknames.p2p");
        assert_eq!(ele.data, "127.0.0.1 yolo");
        assert_eq!(ele.ttl, 10);

        let ele = parsed.records.get(1).unwrap();
        assert_eq!(ele.typ, "TXT");
        assert_eq!(ele.domain, "test");
        assert_eq!(ele.data, "helloworld");
        assert_eq!(ele.ttl, DEFAULT_TTL);
    }



}
