extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate s3_types as types;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate gpgme;

use failure::{Error, ResultExt};
use std::fs::File;
use std::io::{self, stdin, Read};

pub use types::VaultContext as Context;

#[derive(Deserialize, Debug)]
struct Vault {
    users: Option<Vec<String>>
}

#[derive(Debug, Fail)]
#[fail(display = "The top-level error related to handling the vault.")]
enum VaultError {
    #[fail(display = "Could not open vault configuration file at '{}'", path)]
    InvalidConfigurationFile {
        #[cause] cause: io::Error,
        path: String,
    },
}

impl Vault {
    fn from_file(path: &str) -> Result<Vault, Error> {
        let reader: Box<Read> = if path == "-" {
            Box::new(stdin())
        } else {
            Box::new(
                File::open(path).map_err(|cause| VaultError::InvalidConfigurationFile {
                    cause,
                    path: path.to_owned(),
                })?,
            )
        };
        serde_yaml::from_reader(reader).map_err(Into::into)
    }
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: &Context) -> Result<(), Error> {
    Vault::from_file(&ctx.vault_path).context("Could not deserialize vault information")?;
    Ok(())
}
