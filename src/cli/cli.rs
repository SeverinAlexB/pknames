use clap::ArgMatches;

use crate::cli::{lookup::cli_lookup, ls::cli_ls};
use std::path::{PathBuf, Path};

pub fn run_cli() {
    let cmd = clap::Command::new("fancydns")
        .about("A web of trust nslookup replacement.")
        .allow_external_subcommands(true)
        .arg(clap::Arg::new("folder").short('f').long("folder").required(false).help("FancyDns source folder.").default_value("~/.fancydns"))
        .arg(clap::Arg::new("verbose").short('v').long("verbose").required(false).num_args(0).help("Show verbose output."))
        .subcommand(
            clap::Command::new("lookup")
            .about("Lookup the pubkey of a domain.")
            .arg(clap::Arg::new("domain").required(true).help("Domain to resolve. For example: example.com."))
        )
        .subcommand(
            clap::Command::new("ls")
            .about("List your follow lists."),
        );
    let matches = cmd.get_matches();
    let verbose: bool = *matches.get_one("verbose").unwrap();

    let folder_path = validate_folder(&matches);
    if let Err(e) = folder_path {
        println!("Folder validation failed: {}", e);
        return;
    };

    let folder_buf = folder_path.unwrap();
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
        _ => {
            // Default command
            cli_lookup(&matches, folder_buf, verbose);
        },
    };

}


fn validate_folder(matches: &ArgMatches) -> Result<PathBuf, String> {
    let input: &String = matches.get_one("folder").unwrap();
    let expanded = shellexpand::tilde(input);
    let full_path: String = expanded.into();

    let path = Path::new(&full_path);
    let path_buf = PathBuf::from(path);

    if path_buf.exists() {
        if path_buf.is_dir() {
            return Ok(path_buf);
        } else {
            return Err(format!("Given folder must be a directory, not a file. {}", input))
        }
    };

    // Folder does not exist. Let's check if we can create the folder in the parent directory.
    let parent = path_buf.parent();
    if parent.is_none() {
        return Err(format!("Folder not found. {}", input))
    };
    let parent_buf = PathBuf::from(parent.unwrap());
    if !parent_buf.exists() {
        return Err(format!("Folder not found. {}", input));
    };

    let result = std::fs::create_dir(path);
    if let Err(e) = result {
        return Err(format!("Folder not found. {}", e));
    } else {
        println!("Created folder {}", path.to_str().unwrap());
    };
    Ok(path_buf)
}

#[cfg(test)]
mod tests {

    #[test]
    fn single_test() {

    }

}
