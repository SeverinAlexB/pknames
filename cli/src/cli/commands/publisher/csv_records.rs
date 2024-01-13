use std::{path::{PathBuf, Path}, fs::{File, self, ReadDir}, error::Error, io::Read, net::{Ipv4Addr, Ipv6Addr}, fmt::Display};

use csv::Trim;
use pkarr::{dns::{ResourceRecord, Name, rdata::{RData, CNAME, TXT}, Packet}, SignedPacket, Keypair};

pub struct CsvRecord {
    pub typ: String,
    pub domain: String,
    pub data: String,
    pub ttl: u32
}


const DEFAULT_TTL: u32 = 43200; // 12 hours


impl CsvRecord {
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

impl Display for CsvRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {} {}", self.typ, self.domain, self.data, self.ttl)
    }
}

pub struct CsvRecords {
    pub records: Vec<CsvRecord>
}

impl CsvRecords {
    pub fn from_path(csv_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(csv_path)?;
        Self::from_csv(&content)
    }

    pub fn from_csv(content: &str) -> Result<Self, Box<dyn Error>> {
        let mut list: Vec<CsvRecord> = vec![];
        let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .has_headers(false)
        .comment(Some(b'#'))
        .flexible(true)
        .trim(Trim::All)
        .from_reader(content.as_bytes());
        for result in rdr.records() {
            if result.is_err() {
                break;
            };
            let record = result?;
            if record.len() < 3 {
                continue;
            }


            let typ: String = record[0].parse()?;
            if typ.contains("#") {
                continue;
            }
            let domain: String = record[1].parse()?;
            let data: String = record[2].parse()?;
            let ttl = if record.len() == 4 {
                let ttl_res: Result<u32, _> = record[3].parse();
                let ttl_option = ttl_res.ok();
                ttl_option.unwrap_or(DEFAULT_TTL)
            } else {
                DEFAULT_TTL
            };
            list.push(CsvRecord{
                typ: typ.to_uppercase(),
                domain,
                data, 
                ttl
            })

        };
        Ok(CsvRecords {
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



#[cfg(test)]
mod tests {
    use pkarr::Keypair;

    use crate::cli::commands::publisher::csv_records::CsvRecords;

    fn get_test_keypair() -> Keypair {
        // pk:cb7xxx6wtqr5d6yqudkt47drqswxk57dzy3h7qj3udym5puy9cso
        let secret = "6kfe1u5jyqxg644eqfgk1cp4w9yjzwq51rn11ftysuo6xkpc64by";
        let seed = zbase32::decode_full_bytes_str(secret).unwrap();
        let slice: &[u8; 32] = &seed[0..32].try_into().unwrap();
        let keypair = Keypair::from_secret_key(slice);
        keypair
    }

    #[test]
    fn parse_csv() {
        let csv = "
        # Type, Domain, Data, TTL
        A, pknames.p2p, 127.0.0.1, 10
        TXT, test, helloworld,
        TXT, test, helloworld, a
        ";

        let parsed = CsvRecords::from_csv(csv).unwrap();
        assert_eq!(parsed.records.len(), 2);

        let ele = parsed.records.get(0).unwrap();
        assert_eq!(ele.typ, "A");
        assert_eq!(ele.domain, "pknames.p2p");
        assert_eq!(ele.data, "127.0.0.1");
        assert_eq!(ele.ttl, 10);

        let ele = parsed.records.get(1).unwrap();
        assert_eq!(ele.typ, "TXT");
        assert_eq!(ele.domain, "test");
        assert_eq!(ele.data, "helloworld");
        assert_eq!(ele.ttl, 100);
    }

    #[test]
    fn to_signed_packet() {
        let csv = "
        # Type, Domain, Data, TTL
        A, pknames.p2p, 127.0.0.1, 10
        TXT, test, helloworld,
        ";
        let keypair = get_test_keypair();
        let parsed = CsvRecords::from_csv(csv).unwrap();
        let signed_packet = parsed.to_signed_packet(&keypair).expect("Valid csv");
        let packet = signed_packet.packet();
        assert_eq!(packet.answers.len(), 2)
    }



}
