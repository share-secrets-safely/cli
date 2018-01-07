extern crate glob;
extern crate mktemp;

use util::write_at;

use std::path::PathBuf;

use failure::{err_msg, Error, ResultExt};
use std::fs::File;
use std::io::{BufReader, Write, BufRead};
use vault::{strip_ext, ResetCWD, Vault, GPG_GLOB};
use glob::glob;
use mktemp::Temp;
use error::EncryptionError;
use util::fingerprint_of;
use util::{FingerprintUserId, UserIdFingerprint};
use util::new_context;
use util::extract_at_least_one_secret_key;
use util::{KeylistDisplay, export_key};
use util::KeyDisplay;

impl Vault {
    pub fn recipients_list(&self) -> Result<Vec<String>, Error> {
        let recipients_file_path = self.absolute_path(&self.recipients);
        let rfile = File::open(&recipients_file_path)
            .map(BufReader::new)
            .context(format!(
                "Could not open recipients file at '{}' for reading",
                recipients_file_path.display()
            ))?;
        Ok(rfile.lines().collect::<Result<_, _>>().context(format!(
            "Could not read all recipients from file at '{}'",
            recipients_file_path
                .display()
        ))?)
    }

    pub fn init_recipients(&self, gpg_key_ids: &[String], output: &mut Write) -> Result<(), Error> {
        let unknown_path = PathBuf::from("<unknown>");
        let gpg_keys_dir = self.gpg_keys
            .as_ref()
            .map(|p| self.absolute_path(p))
            .ok_or_else(|| {
                format_err!(
                    "The vault at '{}' does not have a gpg_keys directory configured.",
                    self.vault_path
                        .as_ref()
                        .unwrap_or_else(|| &unknown_path)
                        .display()
                )
            })?;
        let mut gpg_ctx = new_context()?;
        let keys = extract_at_least_one_secret_key(&mut gpg_ctx, gpg_key_ids)?;

        let mut buf = Vec::new();
        for key in keys {
            export_key(&mut gpg_ctx, &gpg_keys_dir, &key, &mut buf)?;
            writeln!(
                output,
                "Exported public key for {}.",
                UserIdFingerprint(&key)
            ).ok();
        }
        Ok(())
    }

    pub fn list_recipients(&self, output: &mut Write) -> Result<(), Error> {
        let mut ctx = new_context()?;
        for key in self.recipient_keys(&mut ctx)? {
            writeln!(output, "{}", FingerprintUserId(&key)).ok();
        }
        Ok(())
    }

    pub fn add_recipients(&self, gpg_key_ids: &[String], output: &mut Write) -> Result<(), Error> {
        let mut gpg_ctx = new_context()?;
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
            for key in &keys {
                let fingerprint = export_key(&mut gpg_ctx, &gpg_keys_dir, key, &mut buf)?;
                writeln!(
                    output,
                    "Exported key '{}' for user {}",
                    fingerprint,
                    KeyDisplay(key)
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
        for recipient in &recipients {
            writeln!(&mut writer, "{}", recipient).context(format!(
                "Failed to write recipient '{}' to file at '{}'",
                recipient,
                recipients_file
                    .display()
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
                    .map_err(|e| {
                        EncryptionError::caused_by(
                            e,
                            format!("Failed to re-encrypt {}.", encrypted_file_path.display()),
                            &mut gpg_ctx,
                            &keys,
                        )
                    })?;
            }
            write_at(&self.absolute_path(&encrypted_file_path))
                .context(format!(
                    "Could not open '{}' to write encrypted data",
                    encrypted_file_path.display()
                ))
                .and_then(|mut w| {
                    w.write_all(&obuf).context(format!(
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
}
