extern crate clap;
#[macro_use] extern crate lazy_static;

use clap::{App, SubCommand, Arg};
use std::process::{Command, Output};
use std::io::Write;
use regex::Regex;

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
        search(query);
//        match search(query) {
//            Ok(results) => println!("{:?}", results),
//            Err(error) => writeln!(std::io::stderr(), "{}", error).unwrap()
//        }
    }
}

fn ensure_requirements() {
    let all_available = (
        check_for_requirement("snap"),
        check_for_requirement("flatpak")
    );

    // TODO: Add this to debug logging
    //    println!("{:?}", all_available);

    match all_available {
        (Ok(_), Ok(_)) => return,
        (snap, flatpak) => {
            if snap.is_err() {
                writeln!(std::io::stderr(), "{}", snap.err().unwrap())
                    .unwrap();
            }
            if flatpak.is_err() {
                writeln!(std::io::stderr(), "{}", flatpak.err().unwrap())
                    .unwrap();
            }

            std::process::exit(1);
        }
    }
}

fn check_for_requirement(required_command: &str) -> Result<Output, String>{
    let result = Command::new("which")
        .arg(required_command)
        .output();

    match &result {
        Ok(output) => {
            if output.stdout.len() == 0 {
                return Err(format!("requirement not found: {}", required_command));
            }
            return Ok(output.clone());
        },
        Err(_) => {
            return Err("Error Running `which`".to_string());
        }
    }
}

#[derive(Debug)]
enum Distributor {
    FLATPAK,
    SNAP,
}

#[derive(Debug)]
struct SearchResult {
    name: String,
    version: String,
    publisher: String,
    source: Distributor,
}

//fn search(name: &str) -> Result<Vec<SearchResult>, String> {
fn search(name: &str)  {
    for result in search_snap(name) {
        match result {
            Ok(snap_result) => {
                println!("name: {}", snap_result.name);
                println!("version: {}", snap_result.version);
                println!("publisher: {}", snap_result.publisher);
                println!("source: {:?}", snap_result.source);
            },
            Err(error)=> {
                writeln!(std::io::stderr(), "{}", error);
            }
        }
    }
}

fn search_snap(name: &str) -> Vec<Result<SearchResult, String>> {
    let snap_result = Command::new("snap")
        .arg("search")
        .arg(name)
        .output().unwrap();

    let std_out_string = String::from_utf8_lossy(&snap_result.stdout);

    return std_out_string.split('\n')
        .map(snap_line_to_result)
        .collect();
}

fn snap_line_to_result(snap_line: &str) -> Result<SearchResult, String> {
    lazy_static! {
        static ref SNAP_LINE_REGEX: Regex = Regex::new(r"^(\w+)\s+([\w\.]+)\s+([^\s\t]+)\s+([^\s\t]+)\s(.+)$").unwrap();
    }

    let capture_group = match SNAP_LINE_REGEX.captures(snap_line) {
        Some(_capture_group) => _capture_group,
        _ => return Err("Can't parse snap line".to_string())
    };

    let name = match capture_group.get(1) {
        Some(name_capture) => name_capture.as_str().to_string(),
        _ => return Err("Can't parse snap line".to_string())
    };

    let version = match capture_group.get(2) {
        Some(version_capture) => version_capture.as_str().to_string(),
        _ => return Err("Can't parse snap line".to_string())
    };

    let publisher = match capture_group.get(3) {
        Some(publisher_capture) => publisher_capture.as_str().to_string(),
        _ => return Err("Can't parse snap line".to_string())
    };

    Ok(SearchResult{
        name,
        publisher,
        version,
        source: Distributor::SNAP
    })
}
