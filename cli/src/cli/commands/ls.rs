use std::path::PathBuf;
use clap::ArgMatches;
use crate::cli::config_directory::main_directory::MainDirectory;


pub fn cli_ls(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let config = MainDirectory::new(folder_path);
    config.create_if_it_does_not_exist().unwrap();

    let lists = config.static_lists_dir.read_lists().expect("Readable directory");
    if lists.len() == 0 {
        println!("No lists found in \"{}\".", config.static_lists_dir.path.to_str().unwrap());
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