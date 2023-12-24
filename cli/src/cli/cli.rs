use clap::ArgMatches;

use crate::cli::commands::{lookup::cli_lookup, ls::cli_ls};
use std::path::{PathBuf, Path};

use super::commands::getinfo::cli_getinfo;

/**
 * Main cli entry function.
 */
pub fn run_cli() {
    let cmd = clap::Command::new("fancydns")
        .about("A web of trust nslookup replacement.")
        .allow_external_subcommands(true)
        .arg(clap::Arg::new("directory").short('d').long("directory").required(false).help("FancyDns source directory.").default_value("~/.fancydns"))
        .arg(clap::Arg::new("verbose").short('v').long("verbose").required(false).num_args(0).help("Show verbose output."))
        .subcommand(
            clap::Command::new("lookup")
            .about("Lookup the pubkey of a domain.")
            .arg(clap::Arg::new("domain").required(true).help("Domain to resolve. For example: example.com."))
        )
        .subcommand(
            clap::Command::new("ls")
            .about("List your follow lists."),
        ).subcommand(
            clap::Command::new("getinfo")
            .about("General information."),
        );
    let matches = cmd.get_matches();
    let verbose: bool = *matches.get_one("verbose").unwrap();

    let directory_path = validate_directory(&matches);
    if let Err(e) = directory_path {
        println!("Directory validation failed: {}", e);
        return;
    };

    let folder_buf = directory_path.unwrap();
    if verbose {
        println!("Use folder {}", folder_buf.as_path().to_str().unwrap());
    }

    match matches.subcommand() {
        Some(("ls", matches)) => {
            cli_ls(matches, folder_buf, verbose);
        },
        Some(("lookup", matches)) => {
            cli_lookup(matches, folder_buf, verbose);
        },
        Some(("getinfo", matches)) => {
            cli_getinfo(matches, folder_buf, verbose);
        },
        _ => {
            // Default command
            cli_lookup(&matches, folder_buf, verbose);
        },
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
            return Err(format!("Given path must be a directory, not a file. {}", input))
        }
    };

    // Folder does not exist. Let's check if we can create the folder in the parent directory.
    let parent = path_buf.parent();
    if parent.is_none() {
        return Err(format!("Directory not found. {}", input))
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

#[cfg(test)]
mod tests {

    #[test]
    fn single_test() {

    }

}
