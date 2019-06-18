use failure::Error;

use std::io::{self, BufRead, Write};

use crate::promptable::Promptable;

#[derive(Clone, Debug, Default)]
pub struct IssueToken {
    name: String,
    ticker: String,
    total_supply: String,
}

impl Promptable for IssueToken {
    fn prompt(stdin: &mut dyn BufRead, stdout: &mut dyn Write) -> Result<Box<Self>, Error> {
        write!(stdout, "name: ")?;
        stdout.flush()?;

        let mut buf = String::new();
        stdin.read_line(&mut buf)?;

        let name = buf.trim().to_owned();

        write!(stdout, "ticker: ")?;
        stdout.flush()?;

        buf = String::new();
        stdin.read_line(&mut buf)?;

        // most endpoints will boil all supplied tickers to uppercase
        let ticker = buf.trim().to_uppercase();

        write!(stdout, "total_supply: ")?;
        stdout.flush()?;

        buf = String::new();
        stdin.read_line(&mut buf)?;

        let total_supply = buf.trim().to_owned();

        // Store the string form for easier hashing, but make sure it is a valid number
        let _parsed: u128 = total_supply.parse()?;

        Ok(Box::new(Self {
            name,
            ticker,
            total_supply,
        }))
    }
}
