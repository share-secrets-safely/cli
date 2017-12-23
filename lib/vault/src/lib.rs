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

mod error;

use std::path::{Path, PathBuf};
use error::{ExportKeysError, IOMode, VaultError};

use gpgme::{Context as GpgContext, Protocol};
use failure::{err_msg, Error};
use std::fs::{File, OpenOptions};
use std::io::{stdin, Read, Write};

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
    gpg_keys: Option<PathBuf>,
    #[serde(default = "recipients_default")] recipients: String,
}

impl Vault {
    fn from_file(path: &Path) -> Result<Vault, VaultError> {
        let reader: Box<Read> = if path == Path::new("-") {
            Box::new(stdin())
        } else {
            Box::new(File::open(path)
                .map_err(|cause| VaultError::from_io_err(cause, path, IOMode::Read))?)
        };
        serde_yaml::from_reader(reader).map_err(|cause| VaultError::Deserialization {
            cause,
            path: path.to_owned(),
        })
    }

    fn to_file(&self, path: &Path) -> Result<(), VaultError> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|cause| VaultError::from_io_err(cause, path, IOMode::Write))?;
        serde_yaml::to_writer(&file, self)
            .map_err(|cause| VaultError::Serialization {
                cause,
                path: path.to_owned(),
            })
            .and_then(|_| {
                writeln!(file).map_err(|cause| VaultError::from_io_err(cause, path, IOMode::Write))
            })
    }
}

fn export_keys_with_signatures(
    _gpg_key_ids: Vec<String>,
    _gpg_keys_dir: &Path,
) -> Result<(), ExportKeysError> {
    Ok(())
}

pub fn init(
    gpg_key_ids: Vec<String>,
    gpg_keys_dir: &Path,
    vault_path: &Path,
) -> Result<String, Error> {
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
                    gpg_keys: Some(gpg_keys_dir.to_owned()),
                    recipients: String::from(".recipients"),
                };
                vault.to_file(vault_path)?;
                export_keys_with_signatures(gpg_key_ids, gpg_keys_dir)?;
                Ok(format!("vault initialized at '{}'", vault_path.display()))
            }
        }
    }
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: Context) -> Result<String, Error> {
    use types::VaultCommand;
    match ctx.command {
        VaultCommand::Init {
            gpg_key_ids,
            gpg_keys_dir,
        } => init(gpg_key_ids, &gpg_keys_dir, &ctx.vault_path),
        VaultCommand::List => {
            Vault::from_file(&ctx.vault_path)?;
            Ok(String::new())
        }
    }
}
