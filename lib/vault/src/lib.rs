#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate gpgme;
extern crate s3_types as types;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use gpgme::{Context as GpgContext, Protocol};
use failure::{err_msg, Error, ResultExt};
use std::fs::File;
use std::io::{self, stdin, Read};

pub use types::VaultContext as Context;

#[derive(Deserialize, Debug)]
struct Vault {
    users: Option<Vec<String>>,
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
pub fn do_it(ctx: Context) -> Result<(), Error> {
    use types::VaultCommand;
    match ctx.command {
        VaultCommand::Init {
            gpg_keyfile_path,
            gpg_key_id,
        } => match (gpg_key_id, gpg_keyfile_path) {
            (None, None) => {
                let mut ctx = GpgContext::from_protocol(Protocol::OpenPgp)?;
                let keys: Vec<_> = ctx.find_secret_keys(&Vec::<String>::new())?
                    .filter_map(Result::ok)
                    .collect();
                match keys.len() {
                    1 => {
                        let _key = &keys[0];
                        Ok(())
                    },
                    0 => Err(err_msg("No existing secret GPG key found. Please create one, or specify a key file.")),
                    x => Err(format_err!("Found {} viable keys, which is ambiguous. Please specify one with the key-id argument.", x)),
                }
            }
            _ => unimplemented!("TBD - handle all cases and return Error otherwise"),
        },
        VaultCommand::List => {
            Vault::from_file(&ctx.vault_path).context("Could not deserialize vault information")?;
            Ok(())
        }
    }
}
