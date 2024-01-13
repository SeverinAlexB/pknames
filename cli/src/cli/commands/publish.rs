use std::path::{PathBuf, Path};
use clap::ArgMatches;
use pknames_core::config_directory::dirs::main_directory::MainDirectory;

use super::publisher::{csv_records::CsvRecords, self, pkarr_publisher::PkarrPublisher};



fn parse_csv_path(matches: &ArgMatches) -> PathBuf {
    let unexpanded_path: &String = matches.get_one("csv_path").unwrap();
    let csv_path_str: String = shellexpand::full(unexpanded_path).expect("Valid shell path.").into();
    let path = Path::new(&csv_path_str);
    PathBuf::from(path)
}




pub fn cli_publish(matches: &ArgMatches, directory: PathBuf, verbose: bool) {
    let csv_path = parse_csv_path(matches);
    let csv_records = CsvRecords::from_path(&csv_path);
    if let Err(e) = csv_records {
        eprintln!("Failed to load csv path '{}'. {}", csv_path.to_str().unwrap(), e.to_string());
        std::process::exit(1);
    };

    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();
    let keypair = dir.read_or_create_keypair();

    let records = csv_records.unwrap();

    let packet_result = records.to_signed_packet(&keypair);

    if let Err(e) = packet_result {
        eprintln!("Failed to parse csv records. {}", e.to_string());
        std::process::exit(1);
    }

    println!("Announce {} records", records.records.len());
    for record in records.records.iter() {
        println!("- {}", record);
    }

    let packet = packet_result.unwrap();
    let publisher = PkarrPublisher::new(packet);

    publisher.run();







}