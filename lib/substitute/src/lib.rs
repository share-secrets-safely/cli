extern crate conv;
extern crate failure;

use failure::Error;
use conv::TryFrom;

use std::path::PathBuf;
use std::fmt;
use std::ffi::OsStr;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Spec;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpecError(pub String);

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for Spec {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl ::std::error::Error for SpecError {
    fn description(&self) -> &str {
        "The spec was invalid."
    }
}

impl<'a> TryFrom<&'a str> for Spec {
    type Err = SpecError;

    fn try_from(_src: &'a str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

pub fn substitute(_data: Option<PathBuf>, _specs: &[Spec], _separator: &OsStr) -> Result<(), Error> {
    Ok(())
}
