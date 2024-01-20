use clap::ArgMatches;

use crate::commands::{lookup::cli_lookup, ls::cli_ls};
use std::path::{Path, PathBuf};

use super::commands::{add::cli_add, getinfo::cli_getinfo, publish::cli_publish, remove::cli_remove, resolve::cli_resolve};

/**
 * Main cli entry function.
 */
pub fn run_cli() {
    let cmd = clap::Command::new("pknames")
        .about("A web of trust system to resolve domain names to pkarr public keys.")
        .arg(
            clap::Arg::new("directory")
                .short('d')
                .long("directory")
                .required(false)
                .help("pknames source directory.")
                .default_value("~/.pknames"),
        )
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .long("verbose")
                .required(false)
                .num_args(0)
                .help("Show verbose output."),
        )
        .subcommand(clap::Command::new("getinfo").about("General information."))
        .subcommand(
            clap::Command::new("lookup")
                .about("Lookup the pubkey of a domain.")
                .arg(
                    clap::Arg::new("ui")
                        .short('u')
                        .long("ui")
                        .required(false)
                        .num_args(0)
                        .help("Show graph in a ui frame."),
                )
                .arg(
                    clap::Arg::new("domain")
                        .required(true)
                        .help("Domain to resolve. For example: example.com."),
                ),
        )
        .subcommand(
            clap::Command::new("ls")
                .about("List your follow lists.")
                .arg(
                    clap::Arg::new("ui")
                        .short('u')
                        .long("ui")
                        .required(false)
                        .num_args(0)
                        .help("Show graph in a ui frame."),
                )
                .arg(
                    clap::Arg::new("domain")
                        .short('d')
                        .long("domain")
                        .required(false)
                        .help("Prune graph by domain."),
                ),
        )
        .subcommand(
            clap::Command::new("add")
                .about("Add a follow to your list.")
                .arg(clap::Arg::new("pubkey").required(true).help("Public key to add."))
                .arg(
                    clap::Arg::new("trust")
                        .required(true)
                        .help("Trust value between -1 and 1."),
                )
                .arg(
                    clap::Arg::new("domain")
                        .required(false)
                        .help("Attribute a domain to this public key."),
                ),
        )
        .subcommand(
            clap::Command::new("remove")
                .about("Remove a follow from your list.")
                .arg(clap::Arg::new("pubkey").required(true).help("Public key to remove."))
                .arg(clap::Arg::new("domain").required(false).help("Attributed domain.")),
        )
        .subcommand(
            clap::Command::new("publish")
                .about("Publish pkarr dns records.")
                .arg(
                    clap::Arg::new("csv_path")
                        .required(false)
                        .help("File path to the dns records csv file.")
                        .default_value("./records.csv"),
                )
                .arg(
                    clap::Arg::new("once")
                        .long("once")
                        .required(false)
                        .num_args(0)
                        .help("File path to the dns records csv file."),
                ),
        )
        .subcommand(
            clap::Command::new("resolve")
                .about("Resolve pkarr dns records.")
                .arg(clap::Arg::new("pubkey").required(false).help("Pkarr public key uri.")),
        );
    let matches = cmd.get_matches();
    let verbose: bool = *matches.get_one("verbose").unwrap();

    let directory_path = validate_directory(&matches);
    if let Err(e) = directory_path {
        println!("Directory validation failed: {}", e);
        std::process::exit(1);
    };

    let folder_buf = directory_path.unwrap();
    if verbose {
        println!("Use folder {}", folder_buf.as_path().to_str().unwrap());
    }

    match matches.subcommand() {
        Some(("ls", matches)) => {
            cli_ls(matches, folder_buf, verbose);
        }
        Some(("lookup", matches)) => {
            cli_lookup(matches, folder_buf, verbose);
        }
        Some(("getinfo", matches)) => {
            cli_getinfo(matches, folder_buf, verbose);
        }
        Some(("add", matches)) => {
            cli_add(matches, folder_buf, verbose);
        }
        Some(("remove", matches)) => {
            cli_remove(matches, folder_buf, verbose);
        }
        Some(("publish", matches)) => {
            cli_publish(matches, folder_buf, verbose);
        }
        Some(("resolve", matches)) => {
            cli_resolve(matches, folder_buf, verbose);
        }
        _ => {
            unimplemented!("command not implemented")
        }
    };
}

/**
 * Extract directory path and make sure it is valid.
 * Creates the directory if it does not exist.
 */
fn validate_directory(matches: &ArgMatches) -> Result<PathBuf, String> {
    let input: &String = matches.get_one("directory").unwrap();
    let expanded = shellexpand::tilde(input);
    let full_path: String = expanded.into();

    let path = Path::new(&full_path);
    let path_buf = PathBuf::from(path);

    if path_buf.exists() {
        if path_buf.is_dir() {
            return Ok(path_buf);
        } else {
            return Err(format!("Given path must be a directory, not a file. {}", input));
        }
    };

    // Folder does not exist. Let's check if we can create the folder in the parent directory.
    let parent = path_buf.parent();
    if parent.is_none() {
        return Err(format!("Directory not found. {}", input));
    };
    let parent_buf = PathBuf::from(parent.unwrap());
    if !parent_buf.exists() {
        return Err(format!("Directory not found. {}", input));
    };

    let result = std::fs::create_dir(path);
    if let Err(e) = result {
        return Err(format!("Directory not found. {}", e));
    } else {
        println!("Created directory {}", path.to_str().unwrap());
    };
    Ok(path_buf)
}
