use std::path::PathBuf;
use clap::ArgMatches;
use pknames_core::config_directory::dirs::main_directory::MainDirectory;


pub fn cli_getinfo(_matches: &ArgMatches, folder_path: PathBuf, _verbose: bool) {
    let config = MainDirectory::new(folder_path);
    config.create_if_it_does_not_exist().unwrap();

    println!("Your public key: {}", config.get_public_key_uri());
    println!();

    println!("Your follows");
    let me_list = config.static_lists_dir.read_list(&config.get_public_key_uri()).expect("Me follow list should exist.");
    println!("{}", me_list);

}