use std::{path::PathBuf, ops::Index};
use clap::ArgMatches;

use crate::cli::{config_directory::main_directory::MainDirectory, follow_list::Follow};

pub fn cli_remove(matches: &ArgMatches, directory: PathBuf, verbose: bool) {
    let pubkey: &String = matches.get_one("pubkey").unwrap();
    let domain = match matches.get_one::<String>("domain"){
        Some(val) => Some(val.to_string()),
        None => None
    };

    println!("Remove {} {:?} from my list", pubkey, domain);

    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();

    let mut me_list = dir.static_lists_dir.read_list(&dir.get_zbase32_public_key()).expect("Me list should exist.");
    let follow = Follow::new(pubkey, 0.0, domain);

    if !me_list.follows.contains(&follow) {
        println!("Follow not found in my list.");
        return
    }

    let index = me_list.follows.iter().position(|x| *x == follow).unwrap();
    me_list.follows.remove(index);

    let result = dir.static_lists_dir.write_list(&dir.get_zbase32_public_key(), me_list);
    match result {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Failed to write list: {}", e)
    };

}