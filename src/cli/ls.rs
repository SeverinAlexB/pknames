use std::path::PathBuf;

use clap::ArgMatches;

pub fn cli_ls(matches: &ArgMatches, folder: PathBuf, verbose: bool) {
    println!("ls cli {:?}", matches)
}