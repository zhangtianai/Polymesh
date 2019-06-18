#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

mod issue_token;
mod promptable;
mod transfer;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;
use log::LevelFilter;

use std::{
    env,
    io::{self, BufRead, BufReader, Read, Write},
};

use issue_token::*;
use promptable::*;
use transfer::*;

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
        .about("A tool for manually preparing compliance tokens for Polymesh API calls")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            SubCommand::with_name("gen")
                .about("Generate a token for the supplied endpoint call")
                .arg(
                    Arg::with_name("ENDPOINT")
                        .help("The endpoint call name")
                        .required(true)
                        .possible_values(&["issue_token", "transfer"])
                        .index(1),
                )
                .arg(
                    Arg::with_name("key")
                        .help("The signing key (hex form) to use for creating the compliance token")
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

/// Takes care of the `gen` subcommand
fn handle_gen(matches: &ArgMatches) -> Result<(), Error> {
    match matches
        .value_of("ENDPOINT")
        .ok_or(format_err!("ENDPOINT not matched"))?
    {
        "issue_token" => {
            debug!("Processing issue_token()");
            handle_issue_token()?;
        }
        "transfer" => {debug!("Processing transfer()"); handle_transfer()?;},
        _other => unreachable!(),
    }

    Ok(())
}

/// `gen issue_token`
fn handle_issue_token() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let prompted = IssueToken::prompt(&mut BufReader::new(stdin), &mut stdout)?;

    debug!("Read IssueToken: {:?}", prompted);

    Ok(())
}

/// `gen transfer`
fn handle_transfer() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let prompted = Transfer::prompt(&mut BufReader::new(stdin), &mut stdout)?;

    debug!("Read Transfer: {:?}", prompted);

    Ok(())
}
