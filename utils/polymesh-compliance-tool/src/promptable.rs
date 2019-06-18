use failure::Error;

use std::io::{BufRead, Write};

/// A trait for reading structs in from command line
pub trait Promptable {
    fn prompt(stdin: &mut dyn BufRead, stdout: &mut dyn Write) -> Result<Box<Self>, Error>;
}
