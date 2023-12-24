use std::path::PathBuf;
use clap::ArgMatches;
use crate::cli::config_directory::main_directory::MainDirectory;


pub fn cli_getinfo(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let config = MainDirectory::new(folder_path);
    config.create_if_it_does_not_exist().unwrap();

    let keypair = config.read_or_create_keypair();

    println!("Your public key: pk:{}", keypair.public_key().to_z32())

}