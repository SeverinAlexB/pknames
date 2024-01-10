use std::path::PathBuf;
use clap::ArgMatches;
use pknames_core::config_directory::dirs::main_directory::MainDirectory;


pub fn cli_getinfo(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let config = MainDirectory::new(folder_path);
    config.create_if_it_does_not_exist().unwrap();

    println!("Your public key: {}", config.get_zbase32_public_key());
    println!();

    let me_list = config.static_lists_dir.read_list(&config.get_zbase32_public_key()).expect("Me follow list should exist.");
    println!("{}", me_list);

}