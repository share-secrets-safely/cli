extern crate failure;
extern crate gpgme;
extern crate s3_types as types;
extern crate serde;
extern crate serde_yaml;

use failure::Error;

pub use types::VaultContext as Context;

struct Vault {}

enum VaultError {
    InvalidConfigurationFormat,
}

impl Vault {
    fn from_file(path: &str) -> Result<Vault, VaultError> {
        Ok(Vault {})
    }
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(_ctx: &Context) -> Result<(), Error> {
    Ok(())
}
