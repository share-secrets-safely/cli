use failure::{Fail, err_msg, Error, ResultExt};
use std::fs::File;
use std::io::{Read, Write};
use vault::{strip_ext, ResetCWD, Vault, GPG_GLOB};
use glob::glob;
use mktemp::Temp;
use error::EncryptionError;
use util::{write_at, fingerprint_of, new_context, UserIdFingerprint, KeyDisplay, KeylistDisplay, export_key};
use std::path::Path;
use std::path::PathBuf;
use gpgme::{self, Key};

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
    fn find_signing_key(&self, ctx: &mut gpgme::Context) -> Result<Key, Error> {
        let recipients_fprs = self.recipients_list().context(
            "A recipients list is needed assure the signing key is in the recipients list.",
        )?;
        if recipients_fprs.is_empty() {
            return Err(err_msg(
                "The recipients list is empty, but you are expected to be on that list.",
            ));
        }
        let no_filter = Vec::<String>::new();
//        for k in ctx.find_keys(&no_filter)?.filter_map(Result::ok) {
//            println!("{}", UserIdFingerprint(&k));
//            println!("has secret: {}", k.has_secret());
//            println!("has primary: {}", k.primary_key().is_some());
//            if let Some(pk) = k.primary_key() {
//                println!("pk: can sign: {}", pk.can_sign());
//            }
//            println!("can sign: {}", k.can_sign());
//            println!("can encrypt: {}", k.can_encrypt());
//            println!("is invalid: {}", k.is_invalid());
//            for sk in k.subkeys() {
//                println!(
//                    "subkey {} can sign: {}, can encrypt: {}, is invalid: {}",
//                    sk.fingerprint().unwrap(),
//                    sk.can_sign(),
//                    sk.can_encrypt(),
//                    sk.is_invalid(),
//                );
//            }
//        }

        ctx.find_secret_keys(&no_filter)?
            .filter_map(Result::ok)
//            .filter(Key::has_secret)
//            .filter(|k| k.can_sign() || k.subkeys().any(|sk| sk.can_sign()))
//            .filter_map(|k| fingerprint_of(&k).map(|fpr| (k, fpr)).ok())
//            .filter_map(|(k, fpr)| if recipients_fprs.iter().any(
//                |rfpr| rfpr == &fpr,
//            )
//            {
//                Some(k)
//            } else {
//                None
//            })
            .next()
            .ok_or_else(|| {
                err_msg("Didn't find a single secret key suitable to sign keys.")
            })
    }

    fn read_fingerprint_file(&self, fpr: &str, gpg_keys_dir: &Path) -> Result<(PathBuf, Vec<u8>), Error> {
        let fpr_path = if fpr.len() == 40 {
            gpg_keys_dir.join(fpr)
        } else {
            let _cwd = ResetCWD::new(gpg_keys_dir)?;
            let glob_pattern = format!("*{}", fpr);
            let matching_paths: Vec<_> = glob(&glob_pattern)
                .expect("valid pattern")
                .filter_map(Result::ok)
                .collect();
            match matching_paths.len() {
                1 => gpg_keys_dir.join(&matching_paths[0]),
                0 => {
                    bail!(
                        "Did not find key file matching glob pattern '{}' in directory '{}'.",
                        glob_pattern,
                        gpg_keys_dir.display()
                    )
                }
                l @ _ => {
                    bail!(
                        "Found {} matching key files for glob pattern '{}' in directory '{}', but expected just one.",
                        l,
                        glob_pattern,
                        gpg_keys_dir.display()
                    )
                }
            }
        };
        let mut buf = Vec::new();
        File::open(&fpr_path)
            .context(format!(
                "Could not open key file '{}' for reading",
                fpr_path.display()
            ))
            .and_then(|mut f| {
                f.read_to_end(&mut buf).context(format!(
                    "Could not read key file at '{}'.",
                    fpr_path.display()
                ))
            })?;
        Ok((fpr_path, buf))
    }

    pub fn add_recipients(&self, gpg_key_ids: &[String], verified: bool, output: &mut Write) -> Result<(), Error> {
        let mut gpg_ctx = new_context()?;
        if !verified {
            let gpg_keys_dir = self.gpg_keys_dir().context(
                "Adding unverified recipients requires you to use a vault that has the `gpg-keys` directory configured",
            )?;
            let imported_gpg_keys_ids = gpg_key_ids
                .iter()
                .map(|s| {
                    valid_fingerprint(&s)
                        .and_then(|fpr| self.read_fingerprint_file(&fpr, &gpg_keys_dir))
                        .and_then(|(fpr_path, kb)| {
                            gpg_ctx.import(kb).map_err(|e| {
                                e.context(format!(
                                    "Could not import key to gpg key database from content of file at '{}'",
                                    fpr_path.display()
                                )).into()
                            })
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

            let signing_key = self.find_signing_key(&mut gpg_ctx).context(
                "Did not manage to find suitable signing key \
            for re-exporting the recipient keys.",
            )?;
            gpg_ctx.add_signer(&signing_key)?;
            for key_fpr_to_sign in imported_gpg_keys_ids {
                let key_to_sign = gpg_ctx.find_key(&key_fpr_to_sign)?;
                gpg_ctx
                    .sign_key(&key_to_sign, key_to_sign.user_ids().filter_map(|u| u.name_raw()).map(|n|n.to_bytes()), None)
                    .context(format_err!(
                        "Could not sign key of recipient {} with signing key {}",
                        key_fpr_to_sign,
                        UserIdFingerprint(&signing_key)
                    ))?;
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
                "Re-encrypted '{}' for new recipients",
                strip_ext(&encrypted_file_path)
            ).ok();
        }
        Ok(())
    }
}
