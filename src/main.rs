#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate prettytable;
extern crate clap;

use clap::{App, SubCommand, Arg};
use std::process::{Command, Output};
use std::io::Write;
use regex::Regex;
use prettytable::{Table, format};
use levenshtein::levenshtein;

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

#[derive(Clone, Debug)]
enum Distributor {
    FLATPAK,
    SNAP,
}

impl ToString for Distributor {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Clone, Debug)]
struct SearchResult {
    name: String,
    version: String,
    publisher: String,
    source: Distributor,
    description: String,
    /// levenshtein distance from query string to result name
    lv_distance: usize,
}

// TODO: Move search into own ... module?
fn search(name: &str)  {
    // filter results, logging errors
    let snap_results = search_snap(name);

    let mut results = snap_results;
    results.sort_by(|a, b| a.lv_distance.partial_cmp(&b.lv_distance)
        .unwrap());

    /*
     *print results table
     */
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .borders(' ')
        .separators(&[format::LinePosition::Top],
                    format::LineSeparator::new(' ', ' ', ' ', ' '))
        .padding(0, 5)
        .build();
    table.set_format(format);

    // add header
    table.add_row(row!["SOURCE", "NAME", "VERSION", "PUBLISHER", "Lv Distance", "Description"]);
    // add result rows
    results.iter().for_each(|result| {
        table.add_row(
            row![result.source, result.name, result.version, result.publisher, result.lv_distance, result.description]
        );
    });

    print!("{}", table.to_string());
}

fn search_snap(name: &str) -> Vec<SearchResult> {
    let snap_result = Command::new("snap")
        .arg("search")
        .arg(name)
        .output().unwrap();

    let std_out_string = String::from_utf8_lossy(&snap_result.stdout);

    let unfiltered_results: Vec<Result<SearchResult, String>> = std_out_string.split('\n')
        .map(|result| snap_line_to_result(result, name))
        .skip(1)
        .collect();

    return filter_search_results(unfiltered_results);
}

///
/// Return the OK results, and log the error-ing lines out
///
fn filter_search_results(results: Vec<Result<SearchResult, String>>) -> Vec<SearchResult> {
    results.iter().fold(vec![], |ok_results, result| {
        match result {
            Ok(ok_result) => [&ok_results[..], &vec![ok_result.clone()][..]].concat(),
            Err(error) => {
                error!("{}", error);
                ok_results
            }
        }
    })
}

///
/// Vector of results, either a struct representing the result,
/// or an error wrapping a line that failed to parse
///
fn snap_line_to_result(snap_line: &str, query_name: &str) -> Result<SearchResult, String> {
    lazy_static! {
        static ref SNAP_LINE_REGEX: Regex = Regex::new(r"^(\w+)\s+([\w\.]+)\s+([^\s\t]+)\s+([^\s\t]+)\s(.+)$").unwrap();
    }

    let capture_group = match SNAP_LINE_REGEX.captures(snap_line) {
        Some(_capture_group) => _capture_group,
        _ => return Err(format!("Couldn't parse: {}", snap_line.to_string()))
    };

    let name = match capture_group.get(1) {
        Some(name_capture) => name_capture.as_str().to_string(),
        _ => return Err(format!("Couldn't get name from line:\n {}", snap_line.to_string()))
    };

    let version = match capture_group.get(2) {
        Some(version_capture) => version_capture.as_str().to_string(),
        _ => return Err(format!("Couldn't get version from line:\n {}", snap_line.to_string()))
    };

    let publisher = match capture_group.get(3) {
        Some(publisher_capture) => publisher_capture.as_str().to_string(),
        _ => return Err(format!("Couldn't get publisher from line:\n {}", snap_line.to_string()))
    };

    let description = match capture_group.get(4) {
        Some(description_capture) => description_capture.as_str().to_string(),
        _ => return Err(format!("Couldn't get description from line:\n {}", snap_line.to_string()))
    };

    Ok(SearchResult{
        name: name.clone(),
        publisher,
        version,
        source: Distributor::SNAP,
        description,
        lv_distance: levenshtein(query_name, &name)
    })
}
