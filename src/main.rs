extern crate clap;

use clap::{App, SubCommand, Arg};

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

    let matches = application
        .get_matches_safe()
        .unwrap();

    if let Some(matches) = matches.subcommand_matches("search") {
        let query = matches.value_of("QUERY").expect("Query string required for search");
        println!("Search for: {}", query);
    }
}
