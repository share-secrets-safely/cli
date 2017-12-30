extern crate s3_types;

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use mktemp::Temp;
use itertools::join;
use gpgme;
use vault::Vault;
use failure::{err_msg, Error, Fail, ResultExt};
use util::UserIdFingerprint;
use util::write_at;
use error::FailExt;
use s3_types::{gpg_output_filename, CreateMode, Destination, VaultSpec, WriteMode};
use error::DecryptError;

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
            .map_err(|e: gpgme::Error| {
                if e.code() == gpgme::Error::NO_SECKEY.code() {
                    Error::from(DecryptError { cause: e })
                } else {
                    e.context("Failed to decrypt data.").into()
                }
            })?;

        w.write_all(&output)
            .context("Could not write out all decrypted data.")?;
        Ok(path_for_decryption)
    }

    pub fn recipient_keys(&self, ctx: &mut gpgme::Context) -> Result<Vec<gpgme::Key>, Error> {
        let recipients = self.recipients_list()?;
        if recipients.is_empty() {
            return Err(format_err!(
                "No recipients found in recipients file at '{}'.",
                self.recipients.display()
            ));
        }
        let keys: Vec<gpgme::Key> = ctx.find_keys(&recipients)
            .context("Could not iterate keys for given recipients")?
            .filter_map(Result::ok)
            .filter(|k| k.can_encrypt())
            .collect();
        if keys.len() != recipients.len() {
            let diff = recipients.len() - keys.len();
            let mut msg = vec![
                if diff > 0 {
                    format!(
                        "Didn't find a key for {} recipients in the gpg database.",
                        diff
                    )
                } else {
                    format!(
                        "Found {} additional keys to encrypt for, which is unexpected.",
                        diff
                    )
                },
            ];
            msg.push("All recipients:".into());
            msg.push(recipients.join(", "));
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
                .context(format!("Failed to encrypt {}.", spec))?;
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
