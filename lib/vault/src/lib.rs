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
pub fn do_it(ctx: Context) -> Result<String, Error> {
    use types::VaultCommand;
    match ctx.command {
        VaultCommand::Init { gpg_key_ids } => {
            let mut gpg_ctx = GpgContext::from_protocol(Protocol::OpenPgp)?;
            let keys: Vec<_> = gpg_ctx
                .find_secret_keys(&gpg_key_ids)?
                .filter_map(Result::ok)
                .collect();
            match keys.len() {
                0 => Err(err_msg(
                    "No existing GPG key found for which you have a secret key. Please create one and try again.",
                )),
                x => {
                    if x > 1 && gpg_key_ids.len() == 0 {
                        Err(format_err!("Found {} viable keys for key-ids {:?}, which is ambiguous. Please specify one with the --gpg-key-id argument.", x, gpg_key_ids))
                    } else {
                        Ok(format!("vault initialized at '{}'", ctx.vault_path))
                    }
                }
            }
        }
        VaultCommand::List => {
            Vault::from_file(&ctx.vault_path).context("Could not deserialize vault information")?;
            Ok(String::new())
        }
    }
}
