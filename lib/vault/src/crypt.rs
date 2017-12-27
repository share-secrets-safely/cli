extern crate s3_types;

use s3_types::VaultSpec;
use vault::Vault;
use failure::{err_msg, Error, ResultExt};
use util::UserIdFingerprint;
use std::io::Write;
use itertools::join;
use gpgme;
use std::path::Path;
use std::fs::File;
use s3_types::gpg_output_filename;
use std::process::Command;
use mktemp::Temp;
use util::write_at;
use std::path::PathBuf;
use s3_types::{FileSuffix, WriteMode};

impl Vault {
    pub fn edit(&self, path: &Path, editor: &Path) -> Result<(), Error> {
        let file = Temp::new_file().context("Could not create tempfile to decrypt to")?;
        let tempfile_path = file.to_path_buf();
        let decrypted_file_path = {
            let mut decrypted_writer = write_at(&tempfile_path)
                .context("Failed to open temporary file for writing decrypted content to")?;
            self.decrypt(path, &mut decrypted_writer)
                .context(format!("Failed to decrypt file at '{}'.", path.display()))?
        };
        let mut running_program = Command::new(editor)
            .arg(&tempfile_path)
            .stdin(::std::process::Stdio::inherit())
            .stdout(::std::process::Stdio::inherit())
            .stderr(::std::process::Stdio::inherit())
            .spawn()
            .context(format!(
                "Failed to start editor program at '{}'",
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
                    dst: decrypted_file_path.canonicalize().context(format!(
                        "Could not obtain absolute path for '{}'",
                        decrypted_file_path.display()
                    ))?,
                },
            ],
            WriteMode::AllowOverwrite,
            FileSuffix::Unchanged,
        ).context("Failed to re-encrypt edited content")?;
        Ok(())
    }

    pub fn decrypt(&self, path: &Path, w: &mut Write) -> Result<PathBuf, Error> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let resolved_absolute_path = self.absolute_path(path);
        let resolved_gpg_path = gpg_output_filename(&resolved_absolute_path)?;
        let (mut input, path_for_decryption) = File::open(&resolved_gpg_path)
            .map(|f| (f, resolved_gpg_path.to_owned()))
            .or_else(|_| {
                File::open(&resolved_absolute_path).map(|f| (f, resolved_absolute_path.to_owned()))
            })
            .context(format!(
                "Could not open input file at '{}' for reading. Tried '{}' as well.",
                resolved_gpg_path.display(),
                resolved_absolute_path.display()
            ))?;
        let mut output = Vec::new();
        ctx.decrypt(&mut input, &mut output)
            .context("Failed to decrypt data")?;

        w.write_all(&output)
            .context("Could not write out all decrypted data.")?;
        Ok(path_for_decryption)
    }

    pub fn encrypt(
        &self,
        specs: &[VaultSpec],
        mode: WriteMode,
        suffix: FileSuffix,
    ) -> Result<String, Error> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let recipients = self.recipients_list()?;
        if recipients.is_empty() {
            return Err(format_err!(
                "No recipients found in recipients file at '{}'",
                self.recipients.display()
            ));
        }
        let keys: Vec<gpgme::Key> = ctx.find_keys(&recipients)
            .unwrap()
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
            msg.push("All recipients found in gpg database".into());
            msg.extend(keys.iter().map(|k| format!("{}", UserIdFingerprint(k))));
            return Err(err_msg(msg.join("\n")));
        }

        let mut encrypted = Vec::new();
        for spec in specs {
            let input = {
                let mut buf = Vec::new();
                spec.open_input()?.read_to_end(&mut buf).context(format!(
                    "Could not read all input from '{}' into buffer",
                    spec.source()
                        .map(|s| format!("{}", s.display()))
                        .unwrap_or_else(|| "<stdin>".into())
                ))?;
                buf
            };
            let mut output = Vec::new();
            ctx.encrypt(&keys, input, &mut output)
                .context(format!("Failed to encrypt {}", spec))?;
            spec.open_output_in(&self.resolved_at, mode, suffix)?
                .write_all(&output)
                .context(format!(
                    "Failed to write all encrypted data to '{}'",
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
