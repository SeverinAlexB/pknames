use std::path::PathBuf;
use chrono::{DateTime, Utc};
use clap::ArgMatches;
use pkarr::{PkarrClient, PublicKey};
use pknames_core::config_directory::dirs::main_directory::MainDirectory;

use super::pkarr_records::{PkarrRecords, PkarrRecord};

fn resolve_pkarr(uri: &str) -> (PkarrRecords, DateTime<Utc>)  {
    let client = PkarrClient::new();
    let pubkey: PublicKey = uri.try_into().expect("Should be valid pkarr public key.");
    let res = client.resolve(pubkey);
    if res.is_none() {
        return (PkarrRecords{records: vec![]}, DateTime::<Utc>::MIN_UTC);
    };
    let signed_packet = res.unwrap();
    let timestamp = chrono::DateTime::from_timestamp((signed_packet.timestamp()/1000000).try_into().unwrap(), 0).unwrap();
    let packet = signed_packet.packet();
    let records: Vec<PkarrRecord> = packet.answers.iter().filter_map(|answer| {
        let answer = answer.clone();
        let parse_result: Result<PkarrRecord, String> = answer.try_into();
        if let Err(e) = parse_result {
            eprintln!("Error parsing record. {}", e);
            return None;
        };
        let record: PkarrRecord = parse_result.unwrap();
        Some(record)
    }).collect();

    (PkarrRecords {
        records
    }, timestamp)
}

fn get_arg_pubkey(matches: &ArgMatches, default_uri: &String) -> Option<PublicKey> {
    let uri_arg: &String = matches.get_one("pubkey").unwrap_or(default_uri);
    let trying: Result<PublicKey, _> = uri_arg.as_str().try_into();
    trying.ok()
}


pub fn cli_resolve(matches: &ArgMatches, directory: PathBuf, _verbose: bool) {
    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();
    let keypair = dir.read_or_create_keypair();
    let default_uri = keypair.to_uri_string();
    let pubkey_opt = get_arg_pubkey(matches, &default_uri);

    if pubkey_opt.is_none() {
        eprintln!("pubkey is not a valid pkarr public key.");
        std::process::exit(1);
    };
    let pubkey = pubkey_opt.unwrap();
    let uri = pubkey.to_uri_string();

    println!("Resolve dns records of {}", uri);
    let (records, timestamp) = resolve_pkarr(&uri);

    for record in records.records.iter() {
        println!("- {}", record);
    }
    println!("Last updated at: {timestamp}");
}
