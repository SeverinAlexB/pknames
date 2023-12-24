use std::path::PathBuf;
use clap::ArgMatches;
use crate::cli::config_directory::main_directory::MainDirectory;


pub fn cli_ls(matches: &ArgMatches, folder_path: PathBuf, verbose: bool) {
    let dir = MainDirectory::new(folder_path);
    dir.create_if_it_does_not_exist().unwrap();

    let lists = dir.static_lists_dir.read_lists().expect("Readable directory");
    if lists.len() == 0 {
        println!("No lists found in \"{}\".", dir.static_lists_dir.path.to_str().unwrap());
        return;
    };

    for list in lists.iter() {
        match list {
            Ok(list) => println!("{}", list),
            Err(e) => {
                println!("{}", e);
            }
        }
        println!("");
    }

    let show_gui: bool = *matches.get_one("gui").unwrap();
    if show_gui {
        let lists = dir.static_lists_dir.read_valid_lists();
        todo!("GUI visualization")
    }
}