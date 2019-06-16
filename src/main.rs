extern crate clap;

use clap::{App, SubCommand, Arg};
use std::process::{Command, Output};
use std::io::Error;
use std::io::Write;

fn main() {
    // search sub-command
    let search_sub_command= SubCommand::with_name("search")
        .about("search for packages across distributions.")
        .arg(
            Arg::with_name("QUERY").takes_value(true)
        );

    let application = App::new("troll")
        .version("0.1.0")
        .about("Utility for finding, installing, and removing universal Linux packages.")
        .author("Ross Harrison")
        .subcommand(search_sub_command);

    ensure_requirements();

    let matches = application
        .get_matches_safe()
        .unwrap();

    if let Some(matches) = matches.subcommand_matches("search") {
        let query = matches.value_of("QUERY").expect("Query string required for search");
        println!("Search for: {}", query);
    }
}

fn ensure_requirements() {
    let all_available = (
        check_for_requirement("snap"),
        check_for_requirement("flatpak")
    );
    println!("{:?}", all_available);

    match all_available {
        (Ok(_), Ok(_)) => return,
        (snap, flatpak) => {
            if snap.is_err() {
                writeln!(std::io::stderr(), "{}", snap.err());
            }
            if flatpak.is_err() {
                writeln!(std::io::stderr(), "{}", flatpak.err());
            }

            std::process::exit(1);
        }
    }
}

fn check_for_requirement(required_command: &str) -> Result<&Output, String>{
    let result = Command::new("which")
        .arg(required_command)
        .output();

    match &result {
        Ok(output) => {
            if output.stdout.len() == 0 {
                return Err(format!("requirement not found: {}", required_command));
            }
            return Ok(output);
        },
        Err(ioErr ) => {
            return Err("Error Running `which`".to_string());
        }
    }
}

fn search(name: &str) {

}
