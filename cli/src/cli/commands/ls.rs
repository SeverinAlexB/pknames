use std::path::PathBuf;

use clap::ArgMatches;

use crate::cli::config_folder::ConfigFolder;

pub fn cli_ls(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let config = ConfigFolder::new(folder_path);
    config.create_if_it_does_not_exist().unwrap();
    let lists = config.read_lists().expect("Readable directory");
    if lists.len() == 0 {
        println!("No lists found in \"{}\".", config.get_lists_path().to_str().unwrap());
    } else {
        for list in lists.iter() {
            match list {
                Ok(list) => println!("{}", list),
                Err(e) => {
                    println!("{}", e);
                }
            }
            println!("");
        }
    };
}