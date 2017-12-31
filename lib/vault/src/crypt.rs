extern crate sheesy_types;

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use mktemp::Temp;
use itertools::join;
use gpgme;
use vault::Vault;
use failure::{err_msg, Error, ResultExt};
use util::UserIdFingerprint;
use util::write_at;
use error::FailExt;
use sheesy_types::{gpg_output_filename, CreateMode, Destination, VaultSpec, WriteMode};
use error::{DecryptionError, EncryptionError};
use util::fingerprint_of;

impl Vault {
    pub fn edit(&self, path: &Path, editor: &Path, mode: &CreateMode) -> Result<String, Error> {
        let file = Temp::new_file().context("Could not create tempfile to decrypt to.")?;
        let tempfile_path = file.to_path_buf();
        let decrypted_file_path = {
            let mut decrypted_writer =
                write_at(&tempfile_path).context("Failed to open temporary file for writing decrypted content to.")?;
            self.decrypt(path, &mut decrypted_writer)
                .context(format!("Failed to decrypt file at '{}'.", path.display()))
                .or_else(|err| match (mode, err.first_cause_of::<io::Error>()) {
                    (&CreateMode::Create, Some(_)) => gpg_output_filename(path).map(|p| self.absolute_path(&p)),
                    _ => Err(err.into()),
                })?
        };
        let mut running_program = Command::new(editor)
            .arg(&tempfile_path)
            .stdin(::std::process::Stdio::inherit())
            .stdout(::std::process::Stdio::inherit())
            .stderr(::std::process::Stdio::inherit())
            .spawn()
            .context(format!(
                "Failed to start editor program at '{}'.",
                editor.display()
            ))?;
        let status = running_program
            .wait()
            .context("Failed to wait for editor to exit.")?;
        if !status.success() {
            return Err(format_err!(
                "Editor '{}' failed. Edit aborted.",
                editor.display()
            ));
        }
        self.encrypt(
            &[
                VaultSpec {
                    src: Some(tempfile_path),
                    dst: decrypted_file_path,
                },
            ],
            WriteMode::AllowOverwrite,
            Destination::Unchanged,
        ).context("Failed to re-encrypt edited content.")?;
        Ok(format!("Edited '{}'.", path.display()))
    }

    pub fn decrypt(&self, path: &Path, w: &mut Write) -> Result<PathBuf, Error> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let resolved_absolute_path = self.absolute_path(path);
        let resolved_gpg_path = gpg_output_filename(&resolved_absolute_path)?;
        let (mut input, path_for_decryption) = File::open(&resolved_gpg_path)
            .map(|f| (f, resolved_gpg_path.to_owned()))
            .or_else(|_| File::open(&resolved_absolute_path).map(|f| (f, resolved_absolute_path.to_owned())))
            .context(format!(
                "Could not open input file at '{}' for reading. Tried '{}' as well.",
                resolved_gpg_path.display(),
                resolved_absolute_path.display()
            ))?;
        let mut output = Vec::new();
        ctx.decrypt(&mut input, &mut output)
            .map_err(|e: gpgme::Error| DecryptionError::caused_by(e, "Failed to decrypt data."))?;

        w.write_all(&output)
            .context("Could not write out all decrypted data.")?;
        Ok(path_for_decryption)
    }

    pub fn recipient_keys(&self, ctx: &mut gpgme::Context) -> Result<Vec<gpgme::Key>, Error> {
        let recipients_fprs = self.recipients_list()?;
        if recipients_fprs.is_empty() {
            return Err(format_err!(
                "No recipients found in recipients file at '{}'.",
                self.recipients.display()
            ));
        }
        let keys: Vec<gpgme::Key> = ctx.find_keys(&recipients_fprs)
            .context("Could not iterate keys for given recipients")?
            .filter_map(Result::ok)
            .filter(|k| k.can_encrypt())
            .collect();
        if keys.len() != recipients_fprs.len() {
            let diff = recipients_fprs.len() - keys.len();
            let mut msg = vec![
                if diff > 0 {
                    let existing_fprs: Vec<_> = keys.iter()
                        .map(|k| fingerprint_of(k))
                        .flat_map(Result::ok)
                        .collect();
                    let missing_fprs = recipients_fprs.iter().fold(Vec::new(), |mut m, f| {
                        if existing_fprs.iter().all(|of| of != f) {
                            m.push(f);
                        }
                        m
                    });
                    let mut msg = format!(
                        "Didn't find the key for {} recipient(s) in the gpg database.{}",
                        diff,
                        match self.gpg_keys.as_ref() {
                            Some(dir) => format!(
                                " This might mean it wasn't imported yet from '{}'.",
                                self.absolute_path(dir).display()
                            ),
                            None => String::new(),
                        }
                    );
                    msg.push_str("\nThe following recipient(s) could not be found in the gpg key database:");
                    for fpr in missing_fprs {
                        msg.push_str("\n");
                        let key_path_info = match self.gpg_keys.as_ref() {
                            Some(dir) => {
                                let key_path = self.absolute_path(dir).join(&fpr);
                                format!(
                                    "{}'{}'",
                                    if key_path.is_file() {
                                        "Import key-file using 'gpg --import "
                                    } else {
                                        "Key-file does not exist at "
                                    },
                                    key_path.display()
                                )
                            }
                            None => "No GPG keys directory".into(),
                        };
                        msg.push_str(&format!("{} ({})", &fpr, key_path_info));
                    }
                    msg
                } else {
                    format!(
                        "Found {} additional keys to encrypt for, which may indicate an unusual recipients specification in the recipients file at '{}'",
                        diff,
                        self.absolute_path(&self.recipients).display()
                    )
                },
            ];
            msg.push("All recipients found in gpg database:".into());
            msg.extend(keys.iter().map(|k| format!("{}", UserIdFingerprint(k))));
            return Err(err_msg(msg.join("\n")));
        }
        Ok(keys)
    }

    pub fn encrypt(&self, specs: &[VaultSpec], mode: WriteMode, dst_mode: Destination) -> Result<String, Error> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let keys = self.recipient_keys(&mut ctx)?;

        let mut encrypted = Vec::new();
        for spec in specs {
            let input = {
                let mut buf = Vec::new();
                spec.open_input()?.read_to_end(&mut buf).context(format!(
                    "Could not read all input from '{}' into buffer.",
                    spec.source()
                        .map(|s| format!("{}", s.display()))
                        .unwrap_or_else(|| "<stdin>".into())
                ))?;
                buf
            };
            let mut output = Vec::new();
            ctx.encrypt(&keys, input, &mut output)
                .map_err(|e: gpgme::Error| {
                    EncryptionError::caused_by(e, "Failed to encrypt data.".into(), &mut ctx, &keys)
                })?;
            spec.open_output_in(&self.resolved_at, mode, dst_mode)?
                .write_all(&output)
                .context(format!(
                    "Failed to write all encrypted data to '{}'.",
                    spec.destination().display(),
                ))?;
            encrypted.push(spec.destination());
        }
        Ok(format!(
            "Added {}.",
            join(encrypted.iter().map(|p| format!("'{}'", p.display())), ", ")
        ))
    }
}
