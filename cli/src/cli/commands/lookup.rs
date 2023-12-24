use std::path::PathBuf;

use clap::ArgMatches;

pub fn cli_lookup(matches: &ArgMatches, folder: PathBuf, verbose: bool) {
    let domain: &String = matches.get_one("domain").unwrap();

    println!("lookup cli {} {} ,{:?}", domain, verbose, folder)
}