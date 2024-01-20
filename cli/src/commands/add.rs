use std::path::PathBuf;
use clap::ArgMatches;
use pknames_core::config_directory::{dirs::main_directory::MainDirectory, follow::Follow};


pub fn cli_add(matches: &ArgMatches, directory: PathBuf, _verbose: bool) {
    let pubkey: &String = matches.get_one("pubkey").unwrap();
    let raw_trust: &String = matches.get_one("trust").unwrap();
    let domain = matches.get_one::<String>("domain").map(|s|s.as_str());
    let trust: f32 = raw_trust.parse().expect("trust should be a valid number.");

    println!("Add {} {} {:?}", pubkey, trust, domain);

    let dir = MainDirectory::new(directory);
    dir.create_if_it_does_not_exist().unwrap();

    let mut me_list = dir.static_lists_dir.read_list(&dir.get_public_key_uri()).expect("Me list should exist.");
    let new_follow = Follow::new(pubkey, trust, domain);
    if me_list.follows.contains(&new_follow) {
        let index = me_list.follows.iter().position(|x| *x == new_follow).unwrap();
        me_list.follows.remove(index);
    }
    me_list.follows.push(new_follow);

    let result = dir.static_lists_dir.write_list(&dir.get_public_key_uri(), me_list);
    match result {
        Ok(_) => println!("Success!"),
        Err(e) => eprintln!("Failed to write list: {}", e)
    };

}