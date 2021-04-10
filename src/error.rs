use std::{fmt, error};

#[derive(Debug)]
pub enum Error {

}

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO") // TODO: !
    }
}
