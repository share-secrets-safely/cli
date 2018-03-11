extern crate failure;

use std::path::PathBuf;
use std::ffi::OsStr;
use failure::Error;

mod spec;

pub use spec::*;

pub fn substitute(_data: Option<PathBuf>, _specs: &[Spec], _separator: &OsStr) -> Result<(), Error> {
    Ok(())
}
