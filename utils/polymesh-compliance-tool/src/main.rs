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
use secp256k1::{Message, Secp256k1, SecretKey, constants::SECRET_KEY_SIZE};

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
                .help("The BLS signing key (hex form) to use for creating the compliance token. A new one is generated if .")
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
    // TODO: Alternatively read and parse a key if supplied on the CLI
    let secp = Secp256k1::new();
    let secret_key_arr: [u8; SECRET_KEY_SIZE] = rand::random();
    let key = SecretKey::from_slice(&secret_key_arr)?;

    match matches
        .value_of("ENDPOINT")
        .ok_or(format_err!("ENDPOINT not matched"))?
    {
        "issue_token" => {
            debug!("Processing issue_token()");
            handle_issue_token(&key)?;
        }
        "transfer" => {
            debug!("Processing transfer()");
            handle_transfer(&key)?;
        }
        _other => unreachable!(),
    }

    Ok(())
}

/// `gen issue_token`
fn handle_issue_token(secret_key: &SecretKey) -> Result<(), Error> {
/*
 *    let stdin = io::stdin();
 *    let mut stdout = io::stdout();
 *    let prompted = IssueToken::prompt(&mut BufReader::new(stdin), &mut stdout)?;
 *
 *    debug!("Read IssueToken: {:?}", prompted);
 *
 *    // Create a hash to sign
 *    let mut hasher = Keccak256::new();
 *
 *    hasher.input(prompted.name.as_str());
 *    hasher.input(prompted.ticker.as_str());
 *    hasher.input(prompted.total_supply.as_str());
 *
 *    let hash = hasher.result();
 *    debug!("Computed hash: {}", hex::encode(hash));
 *
 *    // Sign the computed hash
 *    let signature = Bls::sign(hash.as_slice(), signing_key)?;
 *
 *    println!("Key: {}", hex::encode(signing_key.as_bytes()));
 *    println!("Hash: {}", hex::encode(hash));
 *    println!("Token: {}", hex::encode(signature.as_bytes()));
 */

    Ok(())
}

/// `gen transfer`
fn handle_transfer(secret_key: &SecretKey) -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let prompted = Transfer::prompt(&mut BufReader::new(stdin), &mut stdout)?;

    debug!("Read Transfer: {:?}", prompted);

    // Create a hash to sign
    //let mut hasher = Keccak256::new();

    //hasher.input(prompted.ticker.as_str());
    // TODO: implement a multiple hash scheme for transfer()
    //hasher.input(prompted.from.as_str());
    //hasher.input(prompted.to.as_str());
    //hasher.input(prompted.amount.as_str());

    //let hash = hasher.result();
    //debug!("Computed hash: {}", hex::encode(hash));

    // Sign the computed hash

    //println!("Key: {}", hex::encode(signing_key.as_bytes()));
    //println!("Hash: {}", hex::encode(hash));
    //println!("Token: {}", hex::encode(signature.as_bytes()));

    Ok(())
}
