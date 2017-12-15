extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate gpgme;
extern crate s3_types as types;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use failure::Error;
use std::fs::File;
use std::io;

pub use types::VaultContext as Context;

#[derive(Deserialize)]
struct Vault {}

#[derive(Debug, Fail)]
#[fail(display = "A error related to handling the vault.")]
enum VaultError {
    #[fail(display = "Could not open vault configuration file at '{}'", path)]
    InvalidConfigurationFile {
        #[cause] cause: io::Error,
        path: String,
    },
}

impl Vault {
    fn from_file(path: &str) -> Result<Vault, Error> {
        serde_yaml::from_reader(File::open(path).map_err(|cause| {
            VaultError::InvalidConfigurationFile {
                cause,
                path: path.to_owned(),
            }
        })?).map_err(Into::into)
    }
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: &Context) -> Result<(), Error> {
    Vault::from_file(&ctx.vault_path)?;
    Ok(())
}
