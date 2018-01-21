extern crate sheesy_types;

use failure::{Fail, err_msg, Error, ResultExt};
use std::fs::File;
use std::io::Write;
use util::{ResetCWD, strip_ext};
use base::{Vault, GPG_GLOB};
use glob::glob;
use mktemp::Temp;
use error::EncryptionError;
use util::{write_at, fingerprint_of, new_context, UserIdFingerprint, KeyDisplay, KeylistDisplay, export_key};
use sheesy_types::SigningMode;

fn valid_fingerprint(id: &str) -> Result<&str, Error> {
    if id.len() < 8 || id.len() > 40 {
        return Err(format_err!(
            "Fingerprint '{}' must be between 8 and 40 characters long.",
            id
        ));
    }
    let has_only_fingerprint_chars = id.chars().all(|c| match c {
        'a'...'f' | 'A'...'F' | '0'...'9' => true,
        _ => false,
    });

    if has_only_fingerprint_chars {
        Ok(id)
    } else {
        Err(format_err!(
            "Fingerprint '{}' must only contain characters a-f, A-F and 0-9.",
            id
        ))
    }
}

impl Vault {
    pub fn add_recipients(
        &self,
        gpg_key_ids: &[String],
        sign: SigningMode,
        signing_key_id: Option<&str>,
        output: &mut Write,
    ) -> Result<(), Error> {
        let mut gpg_ctx = new_context()?;
        if let SigningMode::Public = sign {
            let gpg_keys_dir = self.gpg_keys_dir().context(
                "Adding unverified recipients requires you to use a vault that has the `gpg-keys` directory configured",
            )?;
            let imported_gpg_keys_ids = gpg_key_ids
                .iter()
                .map(|s| {
                    valid_fingerprint(&s)
                        .and_then(|fpr| self.read_fingerprint_file(&fpr, &gpg_keys_dir))
                        .and_then(|(fpr_path, kb)| {
                            let res = gpg_ctx.import(kb).map_err(|e| {
                                e.context(format!(
                                    "Could not import key to gpg key database from content of file at '{}'",
                                    fpr_path.display()
                                )).into()
                            });
                            writeln!(
                                output,
                                "Imported recipient key at path '{}'",
                                fpr_path.display()
                            ).ok();
                            res
                        })
                })
                .fold(Ok(Vec::new()), |r, k| match k {
                    Ok(imports) => {
                        r.map(|mut v| {
                            v.extend(imports.imports().filter_map(|i| {
                                i.fingerprint().map(ToOwned::to_owned).ok()
                            }));
                            v
                        })
                    }
                    Err(e) => {
                        match r {
                            Ok(_) => Err(e),
                            r @ Err(_) => r.map_err(|f| format_err!("{}\n{}", e, f)),
                        }
                    }
                })?;
            if imported_gpg_keys_ids.len() < gpg_key_ids.len() {
                panic!(
                    "You should come and take a look at this! It should not be possible \
                        to successfully import less keys than specified."
                )
            }

            {
                let mut extra_keys = imported_gpg_keys_ids.clone();
                extra_keys.retain(|k| !gpg_key_ids.iter().any(|ok| k.ends_with(ok)));
                if !extra_keys.is_empty() {
                    return Err(format_err!(
                        "One of the imported key-files contained more than one recipient.\n\
                This might mean somebody trying to sneak in their key. The offending fingerprints are listed below\n\
                {}",
                        extra_keys.join("\n")
                    ));
                }
            }

            let signing_key = self.find_signing_key(&mut gpg_ctx, signing_key_id)
                .context(
                    "Did not manage to find suitable signing key \
            for re-exporting the recipient keys.",
                )?;
            gpg_ctx.add_signer(&signing_key)?;
            for key_fpr_to_sign in imported_gpg_keys_ids {
                let key_to_sign = gpg_ctx.find_key(&key_fpr_to_sign)?;
                gpg_ctx
                    .sign_key(&key_to_sign, None::<&[u8]>, None)
                    .context(format_err!(
                        "Could not sign key of recipient {} with signing key {}",
                        key_fpr_to_sign,
                        UserIdFingerprint(&signing_key)
                    ))?;
                writeln!(
                    output,
                    "Signed recipients key {} with signing key {}",
                    UserIdFingerprint(&key_to_sign),
                    UserIdFingerprint(&signing_key)
                ).ok();
            }
        }
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

        let secrets_dir = self.secrets_path();
        let files_to_reencrypt: Vec<_> = {
            let _change_cwd = ResetCWD::new(&secrets_dir)?;
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
            write_at(&secrets_dir.join(&encrypted_file_path))
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
                "Re-encrypted '{}' for new recipient(s)",
                strip_ext(&encrypted_file_path).display()
            ).ok();
        }
        Ok(())
    }
}
