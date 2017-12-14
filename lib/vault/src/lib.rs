extern crate s3_types as types;
extern crate failure;
extern crate gpgme;

use failure::Error;

pub use types::VaultContext as Context;

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(_ctx: &Context) -> Result<(), Error> {
    Ok(())
}