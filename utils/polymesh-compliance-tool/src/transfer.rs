use failure::Error;

use std::io::{self, BufRead, Write};

use crate::promptable::Promptable;

#[derive(Clone, Debug, Default)]
pub struct Transfer {
    pub ticker: String,
    pub from: String,
    pub to: String,
    pub amount: u128,
}

impl Promptable for Transfer {
    fn prompt(stdin: &mut dyn BufRead, stdout: &mut dyn Write) -> Result<Box<Self>, Error> {
        write!(stdout, "ticker: ")?;
        stdout.flush()?;

        let mut buf = String::new();
        stdin.read_line(&mut buf)?;

        // endpoints are supposed boil supplied tickers to uppercase
        let ticker = buf.trim().to_uppercase();

        write!(stdout, "from: ")?;
        stdout.flush()?;

        let mut buf = String::new();
        stdin.read_line(&mut buf)?;

        let from = buf.trim().to_owned();

        write!(stdout, "to: ")?;
        stdout.flush()?;

        let mut buf = String::new();
        stdin.read_line(&mut buf)?;

        let to = buf.trim().to_owned();

        write!(stdout, "amount: ")?;
        stdout.flush()?;

        buf = String::new();
        stdin.read_line(&mut buf)?;

        let amount = buf.trim().to_owned().parse()?;

        Ok(Box::new(Self {
            ticker,
            from,
            to,
            amount,
        }))
    }
}
