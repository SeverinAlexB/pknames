use std::path::PathBuf;
use clap::ArgMatches;
use crate::cli::config_directory::main_directory::MainDirectory;


pub fn cli_getinfo(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let config = MainDirectory::new(folder_path);
    config.create_if_it_does_not_exist().unwrap();

    println!("Your public key: {}", config.get_zbase32_public_key())

}