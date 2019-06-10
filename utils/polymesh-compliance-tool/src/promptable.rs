use failure::Error;

use std::io::{BufRead, Write};

pub trait Promptable {
    fn prompt(stdin: &mut dyn BufRead, stdout: &mut dyn Write) -> Result<(), Error>;
}
