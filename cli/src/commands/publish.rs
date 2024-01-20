use std::{path::{PathBuf, Path}, fs::ReadDir};
use chrono::Duration;
use clap::ArgMatches;
use pknames_core::config_directory::dirs::main_directory::MainDirectory;

use super::publisher::{pkarr_records::PkarrRecords, self, pkarr_publisher::PkarrPublisher};



fn parse_tabfile_path(matches: &ArgMatches) -> PathBuf {
    let unexpanded_path: &String = matches.get_one("tabfile_path").unwrap();
    let csv_path_str: String = shellexpand::full(unexpanded_path).expect("Valid shell path.").into();
    let path = Path::new(&csv_path_str);
    PathBuf::from(path)
}




pub fn cli_publish(matches: &ArgMatches, directory: PathBuf, verbose: bool) {
    let interval = Duration::minutes(60);
    let once: bool = *matches.get_one("once").unwrap();
    let tabfile_path = parse_tabfile_path(matches);
    let records_result = PkarrRecords::from_path(&tabfile_path);
    if let Err(e) = records_result {
        eprintln!("Failed to load tabfile '{}'. {}", tabfile_path.to_str().unwrap(), e.to_string());
        std::process::exit(1);
    };
    let records = records_result.unwrap();

    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();
    let keypair = dir.read_or_create_keypair();

    let packet_result = records.to_signed_packet(&keypair);

    if let Err(e) = packet_result {
        eprintln!("Failed to parse tabfile records. {}", e.to_string());
        std::process::exit(1);
    }

    println!("Read {} records from {}", records.records.len(), tabfile_path.to_str().unwrap());
    for record in records.records.iter() {
        println!("- {}", record);
    }
    if once {
        println!("Announce once.");
    } else {
        println!("Announce every {}min. Stop with Ctrl-C...", interval.num_minutes());
    }
    println!();

    let packet = packet_result.unwrap();
    let publisher = PkarrPublisher::new(packet);

    if once {
        publisher.run_once();
    } else {
        publisher.run(interval);
    }








}