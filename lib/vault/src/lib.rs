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
mod util;

use util::write_at;

use std::path::{Path, PathBuf};
use error::{IOMode, VaultError};

use gpgme::{Context as GpgContext, Protocol};
use failure::{err_msg, Error, ResultExt};
use std::fs::{create_dir_all, File};
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
        let mut file =
            write_at(path).map_err(|cause| VaultError::from_io_err(cause, path, IOMode::Write))?;
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

pub fn init(
    gpg_key_ids: Vec<String>,
    gpg_keys_dir: &Path,
    recipients_file: &Path,
    vault_path: &Path,
) -> Result<String, Error> {
    let mut gpg_ctx = GpgContext::from_protocol(Protocol::OpenPgp)?;
    let keys = {
        let mut keys_iter = gpg_ctx.find_secret_keys(&gpg_key_ids)?;
        let keys: Vec<_> = keys_iter.by_ref().collect::<Result<_, _>>()?;

        if keys_iter.finish()?.is_truncated() {
            return Err(format_err!(
                "The key list was truncated unexpectedly, while iterating it"
            ));
        }
        keys
    };

    if keys.len() == 0 {
        return Err(err_msg(
            "No existing GPG key found for which you have a secret key. Please create one and try again.",
        ));
    }

    if keys.len() > 1 && gpg_key_ids.len() == 0 {
        return Err(format_err!(
            "Found {} viable keys for key-ids {:?}, which is ambiguous. \
             Please specify one with the --gpg-key-id argument.",
            keys.len(),
            gpg_key_ids
        ));
    };

    let vault = Vault {
        at: at_default(),
        gpg_keys: Some(gpg_keys_dir.to_owned()),
        recipients: String::from(".gpg-id"),
    };
    vault.to_file(vault_path)?;

    if !gpg_keys_dir.is_dir() {
        create_dir_all(gpg_keys_dir).context(format!(
            "Failed to create directory at '{}' for exporting public gpg keys to.",
            gpg_keys_dir.display()
        ))?;
    }

    gpg_ctx.set_armor(true);

    let mut output = Vec::new();
    let mode = gpgme::ExportMode::empty();
    let mut recipients = write_at(recipients_file).context(format!(
        "Failed to open recipients file at '{}'",
        recipients_file.display()
    ))?;
    for key in keys {
        let key_path = {
            let fingerprint = key.fingerprint().map_err(|e| {
                e.map(Into::into)
                    .unwrap_or(err_msg("Fingerprint extraction failed"))
            })?;
            writeln!(recipients, "{}", fingerprint).context(format!(
                "Could not append fingerprint to file at '{}'",
                recipients_file.display()
            ))?;
            gpg_keys_dir.join(fingerprint)
        };
        gpg_ctx
            .export_keys([key].iter(), mode, &mut output)
            .context(format!(
                "Failed to export at least one public key with signatures."
            ))?;
        write_at(&key_path)
            .and_then(|mut f| f.write_all(&output))
            .context(format!(
                "Could not write public key file at '{}'",
                key_path.display()
            ))?;
        output.clear();
    }
    recipients.flush().context(format!(
        "Failed to flush recipients file at '{}'",
        recipients_file.display()
    ))?;
    Ok(format!("vault initialized at '{}'", vault_path.display()))
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: Context) -> Result<String, Error> {
    use types::VaultCommand;
    match ctx.command {
        VaultCommand::Init {
            gpg_key_ids,
            gpg_keys_dir,
            recipients_file,
        } => init(
            gpg_key_ids,
            &gpg_keys_dir,
            &recipients_file,
            &ctx.vault_path,
        ),
        VaultCommand::List => {
            Vault::from_file(&ctx.vault_path)?;
            Ok(String::new())
        }
    }
}
