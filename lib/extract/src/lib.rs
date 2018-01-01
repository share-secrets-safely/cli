extern crate failure;
extern crate sheesy_types as types;

use failure::Error;

pub use types::ExtractionContext as Context;

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(_ctx: Context) -> Result<(), Error> {
    Ok(())
}
