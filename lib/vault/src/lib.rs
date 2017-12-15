extern crate failure;
extern crate gpgme;
extern crate s3_types as types;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use failure::{Error, ResultExt};
use std::fs::File;

pub use types::VaultContext as Context;

#[derive(Deserialize)]
struct Vault {}

impl Vault {
    // TODO use failure_derive and see how causes work for us
    fn from_file(path: &str) -> Result<Vault, Error> {
        serde_yaml::from_reader(File::open(path).context(format!(
            "Could not open vault configuration file at '{}'",
            path
        ))?).map_err(Into::into)
    }
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: &Context) -> Result<(), Error> {
    Vault::from_file(&ctx.vault_path)?;
    Ok(())
}
