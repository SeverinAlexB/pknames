use std::path::{PathBuf, Path};
use clap::ArgMatches;
use pknames_core::config_directory::{dirs::main_directory::MainDirectory, follow::Follow};


fn parse_csv_path(matches: &ArgMatches) -> PathBuf {
    let unexpanded_path: &String = matches.get_one("csv_path").unwrap();
    let csv_path_str: String = shellexpand::full(unexpanded_path).expect("Valid shell path.").into();
    let path = Path::new(&csv_path_str);
    PathBuf::from(path)
}




pub fn cli_publish(matches: &ArgMatches, directory: PathBuf, verbose: bool) {
    let csv_path = parse_csv_path(matches);
    let csv_result = csv::Reader::from_path(csv_path.clone());

    if csv_result.is_err() {
        let error = csv_result.unwrap_err();
        eprintln!("Failed to read csv file {:?}. {}", csv_path, error.to_string());
        std::process::exit(1);
    }

    // let reader = csv_result.unwrap();
    // if reader.records().map(|line| {
    //     line.
    // })


    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();


}