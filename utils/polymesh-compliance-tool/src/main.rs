#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

mod promptable;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;
use log::LevelFilter;

use std::env;

use promptable::*;

fn main() -> Result<(), Error> {
    // Init logging
    match env::var("RUST_LOG") {
        Ok(_) => env_logger::init(),
        Err(_) => {
            env_logger::Builder::new()
                .filter_level(LevelFilter::Info)
                .init();
        }
    }

    let matches = App::new("The Polymesh Compliance Token Tool")
        .author("Polymath Network")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            SubCommand::with_name("gen")
                .about("Generate a token for the supplied endpoint call")
                .arg(
                    Arg::with_name("ENDPOINT")
                        .help("The endpoint call name")
                        .required(true)
                        .possible_values(&["issue_token, transfer"])
                        .index(0),
                )
                .arg(
                    Arg::with_name("key")
                        .help("The signing key to use for creating the compliance token")
                        .short("k")
                        .long("key"),
                ),
        )
        .get_matches();

    match matches.subcommand_matches("gen") {
        Some(matches) => handle_gen(matches)?,
        None => unreachable!(),
    }

    Ok(())
}

fn handle_gen(matches: &ArgMatches) -> Result<(), Error> {

    match matches
        .value_of("ENDPOINT")
        .ok_or(format_err!("ENDPOINT not matched"))?
    {
        "issue_token" => debug!("Processing issue_token()"),
        "transfer" => debug!("Processing transfer()"),
        _other => unreachable!(),
    }

    Ok(())
}
