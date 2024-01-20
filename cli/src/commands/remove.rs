use std::path::PathBuf;
use clap::ArgMatches;
use pknames_core::config_directory::{dirs::main_directory::MainDirectory, follow::Follow};


pub fn cli_remove(matches: &ArgMatches, directory: PathBuf, _verbose: bool) {
    let pubkey: &String = matches.get_one("pubkey").unwrap();
    let domain = matches.get_one::<String>("domain");

    println!("Remove {} {:?} from my list", pubkey, domain);

    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();

    let mut me_list = dir.static_lists_dir.read_list(&dir.get_public_key_uri()).expect("Me list should exist.");
    let follow = Follow::new(pubkey, 0.0, domain.map(|s|s.as_str()));

    if !me_list.follows.contains(&follow) {
        eprintln!("Follow not found in my list.");
        return
    }

    let index = me_list.follows.iter().position(|x| *x == follow).unwrap();
    me_list.follows.remove(index);

    let result = dir.static_lists_dir.write_list(&dir.get_public_key_uri(), me_list);
    match result {
        Ok(_) => println!("Success!"),
        Err(e) => {
            eprintln!("Failed to write list: {}", e);
            std::process::exit(1)
        }
    };

}