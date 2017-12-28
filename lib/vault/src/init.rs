use gpgme;
use util::write_at;

use std::path::Path;

use itertools::join;
use failure::{err_msg, Error, ResultExt};
use std::fs::create_dir_all;
use std::io::Write;
use std::fmt;
use vault::Vault;

struct KeylistDisplay<'a>(&'a Vec<gpgme::Key>);

impl<'a> fmt::Display for KeylistDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            join(
                self.0
                    .iter()
                    .flat_map(|k| k.user_ids())
                    .map(|u| u.id().unwrap_or("[none]")),
                ", "
            )
        )
    }
}

impl Vault {
    pub fn add_recipients(&self, gpg_key_ids: &[String]) -> Result<String, Error> {
        let mut gpg_ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let keys = {
            let mut keys_iter = gpg_ctx.find_keys(gpg_key_ids)?;
            let keys: Vec<_> = keys_iter.by_ref().collect::<Result<_, _>>()?;

            if keys_iter.finish()?.is_truncated() {
                return Err(err_msg(
                    "The key list was truncated unexpectedly, while iterating it",
                ));
            }
            keys
        };
        if keys.len() != gpg_key_ids.len() {
            return Err(format_err!(
                "Found {} viable keys for key-ids ({}), for {} given user ids.",
                keys.len(),
                KeylistDisplay(&keys),
                gpg_key_ids.len()
            ));
        };

        let output = Vec::<String>::new();
        if let Some(gpg_keys_dir) = self.gpg_keys.as_ref() {
            let gpg_keys_dir = self.absolute_path(gpg_keys_dir);
        }

        let recipients_file = self.absolute_path(&self.recipients);
        Ok(output.join("\n"))
    }

    pub fn init(
        gpg_key_ids: &[String],
        gpg_keys_dir: &Path,
        recipients_file: &Path,
        vault_path: &Path,
        name: Option<String>,
    ) -> Result<Self, Error> {
        let vault = Vault {
            gpg_keys: Some(gpg_keys_dir.to_owned()),
            recipients: recipients_file.to_owned(),
            resolved_at: vault_path
                .parent()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format_err!("The vault directory '{}' is invalid.", vault_path.display()))?,
            name,
            ..Default::default()
        };

        let mut gpg_ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let keys = {
            let mut keys_iter = gpg_ctx.find_secret_keys(gpg_key_ids)?;
            let keys: Vec<_> = keys_iter.by_ref().collect::<Result<_, _>>()?;

            if keys_iter.finish()?.is_truncated() {
                return Err(err_msg(
                    "The key list was truncated unexpectedly, while iterating it",
                ));
            }
            keys
        };

        if keys.is_empty() {
            return Err(err_msg(
                "No existing GPG key found for which you have a secret key. Please create one and try again.",
            ));
        }

        if keys.len() > 1 && gpg_key_ids.is_empty() {
            return Err(format_err!(
                "Found {} viable keys for key-ids ({}), which is ambiguous. \
                 Please specify one with the --gpg-key-id argument.",
                keys.len(),
                KeylistDisplay(&keys)
            ));
        };
        vault.to_file(vault_path)?;

        let gpg_keys_dir = vault.absolute_path(gpg_keys_dir);
        let recipients_file = vault.absolute_path(recipients_file);
        if gpg_keys_dir.is_dir() {
            let num_entries = gpg_keys_dir
                .read_dir()
                .context(format!(
                    "Failed to open directory '{}' to see if it is empty.",
                    gpg_keys_dir.display()
                ))?
                .count();
            if num_entries > 0 {
                return Err(format_err!(
                    "Cannot export keys into existing, non-empty directory at '{}'",
                    gpg_keys_dir.display()
                ));
            }
        } else {
            create_dir_all(&gpg_keys_dir).context(format!(
                "Failed to create directory at '{}' for exporting public gpg keys to.",
                gpg_keys_dir.display()
            ))?;
        }

        gpg_ctx.set_armor(true);

        let mut output = Vec::new();
        let mode = gpgme::ExportMode::empty();
        if recipients_file.is_file() {
            return Err(format_err!(
                "Cannot write recipients into existing file at '{}'",
                recipients_file.display()
            ));
        }
        let mut recipients = write_at(&recipients_file).context(format!(
            "Failed to open recipients file at '{}'",
            recipients_file.display()
        ))?;
        for key in keys {
            let key_path = {
                let fingerprint = key.fingerprint().map_err(|e| {
                    e.map(Into::into)
                        .unwrap_or_else(|| err_msg("Fingerprint extraction failed"))
                })?;
                writeln!(recipients, "{}", fingerprint).context(format!(
                    "Could not append fingerprint to file at '{}'",
                    recipients_file.display()
                ))?;
                gpg_keys_dir.join(fingerprint)
            };
            gpg_ctx
                .export_keys([key].iter(), mode, &mut output)
                .context(err_msg(
                    "Failed to export at least one public key with signatures.",
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
        Ok(vault)
    }
}
