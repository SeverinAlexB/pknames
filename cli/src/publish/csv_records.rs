use std::{path::{PathBuf, Path}, fs::{File, self, ReadDir}, error::Error, io::Read, net::Ipv4Addr};

use csv::Trim;
use pkarr::{dns::{ResourceRecord, Name, rdata::RData, Packet}, SignedPacket, Keypair};

struct CsvRecord {
    typ: String,
    domain: String,
    data: String,
    ttl: u32
}


impl CsvRecord {


    pub fn to_resource_record<'a>(&'a self) -> Result<ResourceRecord<'a>, Box<dyn Error>> {
        let name: Name<'a> = Name::try_from(self.domain.as_str())?;
        let rdata = match self.typ.as_str() {
            "A" => {
                let ip: Ipv4Addr = self.data.parse()?;
                RData::A(ip.into())
            },
            _ => {
                todo!("To be implemented record type")
            }
        };
        Ok(ResourceRecord::new(name, pkarr::dns::CLASS::IN, self.ttl, rdata))
    }
}

struct CsvRecords {
    records: Vec<CsvRecord>
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
            println!("{:?}", record);

            if record.len() < 3 {
                continue;
            }

            let typ: String = record[0].parse()?;
            let domain: String = record[1].parse()?;
            let data: String = record[2].parse()?;
            let ttl = record[1].parse().ok().unwrap_or(100);
            list.push(CsvRecord{
                typ,
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



// impl<'a> TryInto<ResourceRecord<'a>> for CsvRecord {
//     type Error = Box<dyn Error>;

//     fn try_into(self) -> Result<ResourceRecord<'a>, Box<dyn Error>> {
//         let name = Name::try_from(self.domain.clone().as_str())?;
//         let rdata = match self.typ.as_str() {
//             "A" => {
//                 let ip: Ipv4Addr = self.data.parse()?;
//                 RData::A(ip.into())
//             },
//             _ => {
//                 todo!("To be implemented record type")
//             }
//         };
//         Ok(ResourceRecord::new(name, pkarr::dns::CLASS::IN, self.ttl, rdata))
//     }
// }



#[cfg(test)]
mod tests {
    use crate::publish::csv_records::CsvRecords;

    #[test]
    fn parse_csv() {
        let csv = "
        # Type, Domain, Data, TTL
        A, pknames.p2p, 127.0.0.1, 100
        TXT, test, helloworld, 300
        ";

        let parsed = CsvRecords::from_csv(csv);
        assert!(parsed.is_ok());
    }



}
