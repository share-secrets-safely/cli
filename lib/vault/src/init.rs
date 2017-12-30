extern crate glob;
extern crate mktemp;

use gpgme;
use util::write_at;

use std::path::Path;

use itertools::join;
use failure::{err_msg, Error, ResultExt};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::fmt;
use vault::{strip_ext, ResetCWD, Vault, GPG_GLOB};
use glob::glob;
use mktemp::Temp;

struct KeylistDisplay<'a>(&'a [gpgme::Key]);

impl<'a> fmt::Display for KeylistDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", join(self.0.iter().map(|k| KeyDisplay(k)), ", "))
    }
}
struct KeyDisplay<'a>(&'a gpgme::Key);

impl<'a> fmt::Display for KeyDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            join(self.0.user_ids().map(|u| u.id().unwrap_or("[none]")), ", ")
        )
    }
}

fn fingerprint_of(key: &gpgme::Key) -> Result<String, Error> {
    key.fingerprint()
        .map_err(|e| {
            e.map(Into::into)
                .unwrap_or_else(|| err_msg("Fingerprint extraction failed"))
        })
        .map(ToOwned::to_owned)
}

fn export_key(
    ctx: &mut gpgme::Context,
    gpg_keys_dir: &Path,
    key: &gpgme::Key,
    buf: &mut Vec<u8>,
) -> Result<String, Error> {
    let fingerprint = fingerprint_of(key)?;
    let key_path = gpg_keys_dir.join(&fingerprint);
    ctx.set_armor(true);
    ctx.export_keys(
        [key].iter().map(|k| *k),
        gpgme::ExportMode::empty(),
        &mut *buf,
    ).context(err_msg(
        "Failed to export at least one public key with signatures.",
    ))?;
    write_at(&key_path)
        .and_then(|mut f| f.write_all(buf))
        .context(format!(
            "Could not write public key file at '{}'",
            key_path.display()
        ))?;
    buf.clear();
    Ok(fingerprint.to_owned())
}

impl Vault {
    pub fn add_recipients(&self, gpg_key_ids: &[String], output: &mut Write) -> Result<(), Error> {
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

        if let Some(gpg_keys_dir) = self.gpg_keys.as_ref() {
            let gpg_keys_dir = self.absolute_path(gpg_keys_dir);
            let mut buf = Vec::new();
            for key in keys.iter() {
                let fingerprint = export_key(&mut gpg_ctx, &gpg_keys_dir, key, &mut buf)?;
                writeln!(
                    output,
                    "Exported key '{}' for user {}",
                    fingerprint,
                    KeyDisplay(&key)
                ).ok();
            }
        }

        let mut recipients = self.recipients_list()?;
        for key in keys {
            recipients.push(fingerprint_of(&key)?);
            writeln!(output, "Added recipient {}", KeyDisplay(&key)).ok();
        }
        recipients.sort();
        recipients.dedup();

        let recipients_file = self.absolute_path(&self.recipients);
        let mut writer = write_at(&recipients_file).context(format!(
            "Failed to open recipients at '{}' file for writing",
            recipients_file.display()
        ))?;
        for recipient in recipients.iter() {
            writeln!(&mut writer, "{}", recipient).context(format!(
                "Failed to write recipient '{}' to file at '{}'",
                recipient,
                recipients_file.display()
            ))?
        }

        let keys = self.recipient_keys(&mut gpg_ctx)?;

        let mut obuf = Vec::new();

        let files_to_reencrypt: Vec<_> = {
            let _change_cwd = ResetCWD::new(&self.resolved_at)?;
            glob(GPG_GLOB)
                .expect("valid pattern")
                .filter_map(Result::ok)
                .collect()
        };
        for encrypted_file_path in files_to_reencrypt {
            let tempfile = Temp::new_file().context(format!(
                "Failed to create temporary file to hold decrypted '{}'",
                encrypted_file_path.display()
            ))?;
            {
                let mut plain_writer = write_at(&tempfile.to_path_buf())?;
                self.decrypt(&encrypted_file_path, &mut plain_writer)
                    .context(format!(
                        "Could not decrypt '{}' to re-encrypt for new recipients.",
                        encrypted_file_path.display()
                    ))?;
            }
            {
                let mut plain_reader = File::open(tempfile.to_path_buf())?;
                gpg_ctx
                    .encrypt(&keys, &mut plain_reader, &mut obuf)
                    .context(format!(
                        "Failed to re-encrypt {}.",
                        encrypted_file_path.display()
                    ))?;
            }
            write_at(&self.absolute_path(&encrypted_file_path))
                .context(format!(
                    "Could not open '{}' to write encrypted data",
                    encrypted_file_path.display()
                ))
                .and_then(|mut w| {
                    w.write_all(&mut obuf).context(format!(
                        "Failed to write out encrypted data to '{}'",
                        encrypted_file_path.display()
                    ))
                })?;

            obuf.clear();
            writeln!(
                output,
                "Re-encrypted '{}' for new recipients",
                strip_ext(&encrypted_file_path)
            ).ok();
        }
        Ok(())
    }

    pub fn init(
        at: &Path,
        gpg_key_ids: &[String],
        gpg_keys_dir: &Path,
        recipients_file: &Path,
        vault_path: &Path,
        name: Option<String>,
    ) -> Result<Self, Error> {
        let vault = Vault {
            gpg_keys: Some(gpg_keys_dir.to_owned()),
            recipients: recipients_file.to_owned(),
            name,
            at: at.to_owned(),
            ..Default::default()
        }.set_resolved_at(vault_path)?;

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
                "No existing GPG key found for which you have a secret key. \
                 Please create one with 'gpg --gen-key' and try again.",
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

        let mut output = Vec::new();
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
            let fingerprint = export_key(&mut gpg_ctx, &gpg_keys_dir, &key, &mut output)?;

            writeln!(recipients, "{}", fingerprint).context(format!(
                "Could not append fingerprint to file at '{}'",
                recipients_file.display()
            ))?;
        }
        recipients.flush().context(format!(
            "Failed to flush recipients file at '{}'",
            recipients_file.display()
        ))?;
        Ok(vault)
    }
}
