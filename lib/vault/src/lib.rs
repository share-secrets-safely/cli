#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate gpgme;
extern crate s3_types as types;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use gpgme::{Context as GpgContext, Protocol};
use failure::{err_msg, Error};
use std::fs::{File, OpenOptions};
use std::io::{self, stdin, Read, Write};

pub use types::VaultContext as Context;

fn recipients_default() -> String {
    String::from(".gpg-id")
}

fn at_default() -> String {
    String::from(".")
}

#[derive(Deserialize, Serialize, Debug)]
struct Vault {
    #[serde(default = "at_default")] at: String,
    gpg_keys: Option<String>,
    #[serde(default = "recipients_default")] recipients: String,
}

#[derive(Debug, Fail)]
#[fail(display = "The top-level error related to handling the vault.")]
enum VaultError {
    #[fail(display = "Could not access vault configuration file for reading '{}'", path)]
    ReadFile {
        #[cause] cause: io::Error,
        path: String,
    },
    #[fail(display = "Could not open vault configuration file for writing '{}'", path)]
    WriteFile {
        #[cause] cause: io::Error,
        path: String,
    },
    #[fail(display = "Could not deserialize vault configuration file at '{}'", path)]
    Deserialization {
        #[cause] cause: serde_yaml::Error,
        path: String,
    },
    #[fail(display = "Could not serialize vault configuration file to '{}'", path)]
    Serialization {
        #[cause] cause: serde_yaml::Error,
        path: String,
    },
}

enum IO {
    Read,
    Write,
}

impl VaultError {
    fn from_io_err(cause: io::Error, path: &str, mode: IO) -> Self {
        match mode {
            IO::Write => VaultError::WriteFile {
                cause,
                path: path.to_owned(),
            },
            IO::Read => VaultError::ReadFile {
                cause,
                path: path.to_owned(),
            },
        }
    }
}

impl Vault {
    fn from_file(path: &str) -> Result<Vault, VaultError> {
        let reader: Box<Read> = if path == "-" {
            Box::new(stdin())
        } else {
            Box::new(File::open(path)
                .map_err(|cause| VaultError::from_io_err(cause, path, IO::Read))?)
        };
        serde_yaml::from_reader(reader).map_err(|cause| VaultError::Deserialization {
            cause,
            path: path.to_owned(),
        })
    }

    fn to_file(&self, path: &str) -> Result<(), VaultError> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|cause| VaultError::from_io_err(cause, path, IO::Write))
            .and_then(|mut w| {
                serde_yaml::to_writer(&w, self)
                    .map_err(|cause| VaultError::Serialization {
                        cause,
                        path: path.to_owned(),
                    })
                    .and_then(|_| {
                        writeln!(w).map_err(|cause| VaultError::from_io_err(cause, path, IO::Write))
                    })
            })
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
                        let vault = Vault {
                            at: at_default(),
                            gpg_keys: Some(String::from(".gpg-keys")),
                            recipients: String::from(".recipients"),
                        };
                        vault.to_file(&ctx.vault_path)?;
                        Ok(format!("vault initialized at '{}'", ctx.vault_path))
                    }
                }
            }
        }
        VaultCommand::List => {
            Vault::from_file(&ctx.vault_path)?;
            Ok(String::new())
        }
    }
}
