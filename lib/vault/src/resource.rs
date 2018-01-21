use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs::{remove_file, File};
use std::process::Command;

use mktemp::Temp;
use itertools::join;
use gpgme;
use base::Vault;
use failure::{Error, ResultExt};
use error::FailExt;
use sheesy_types::{gpg_output_filename, CreateMode, Destination, VaultSpec, WriteMode};
use error::{DecryptionError, EncryptionError};
use util::{write_at, strip_ext, new_context};

impl Vault {
    pub fn edit(&self, path: &Path, editor: &Path, mode: &CreateMode, output: &mut Write) -> Result<(), Error> {
        let file = Temp::new_file().context(
            "Could not create tempfile to decrypt to.",
        )?;
        let tempfile_path = file.to_path_buf();
        let decrypted_file_path = {
            let mut decrypted_writer = write_at(&tempfile_path).context(
                "Failed to open temporary file for writing decrypted content to.",
            )?;
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
        let status = running_program.wait().context(
            "Failed to wait for editor to exit.",
        )?;
        if !status.success() {
            return Err(format_err!(
                "Editor '{}' failed. Edit aborted.",
                editor.display()
            ));
        }
        let mut zero = Vec::new();
        self.encrypt(
            &[
                VaultSpec {
                    src: Some(tempfile_path),
                    dst: decrypted_file_path,
                },
            ],
            WriteMode::AllowOverwrite,
            Destination::Unchanged,
            &mut zero,
        ).context("Failed to re-encrypt edited content.")?;
        writeln!(output, "Edited '{}'.", path.display()).ok();
        Ok(())
    }

    pub fn decrypt(&self, path: &Path, w: &mut Write) -> Result<PathBuf, Error> {
        let mut ctx = new_context()?;
        let resolved_absolute_path = self.secrets_path().join(path);
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
        ctx.decrypt(&mut input, &mut output).map_err(
            |e: gpgme::Error| {
                DecryptionError::caused_by(e, "Failed to decrypt data.")
            },
        )?;

        w.write_all(&output).context(
            "Could not write out all decrypted data.",
        )?;
        Ok(path_for_decryption)
    }

    pub fn remove(&self, specs: &[VaultSpec], output: &mut Write) -> Result<(), Error> {
        let sp = self.secrets_path();
        for spec in specs {
            let path = {
                let gpg_path = spec.output_in(&sp, Destination::ReolveAndAppendGpg)?;
                if gpg_path.exists() {
                    gpg_path
                } else {
                    let mut new_path = strip_ext(&gpg_path);
                    if !new_path.exists() {
                        return Err(format_err!("No file present at '{}'", gpg_path.display()));
                    }
                    new_path
                }
            };
            remove_file(&path).context(format!(
                "Failed to remove file at '{}'.",
                path.display()
            ))?;
            writeln!(output, "Removed file at '{}'", path.display()).ok();
        }
        Ok(())
    }

    pub fn encrypt(
        &self,
        specs: &[VaultSpec],
        mode: WriteMode,
        dst_mode: Destination,
        output: &mut Write,
    ) -> Result<(), Error> {
        let mut ctx = new_context()?;
        let keys = self.recipient_keys(&mut ctx)?;

        let mut encrypted = Vec::new();
        let secrets_path = self.secrets_path();
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
            let mut encrypted_bytes = Vec::new();
            ctx.encrypt(&keys, input, &mut encrypted_bytes).map_err(
                |e: gpgme::Error| EncryptionError::caused_by(e, "Failed to encrypt data.".into(), &mut ctx, &keys),
            )?;
            spec.open_output_in(&secrets_path, mode, dst_mode, output)?
                .write_all(&encrypted_bytes)
                .context(format!(
                    "Failed to write all encrypted data to '{}'.",
                    spec.destination().display(),
                ))?;
            encrypted.push(spec.destination());
        }
        writeln!(
            output,
            "Added {}.",
            join(encrypted.iter().map(|p| format!("'{}'", p.display())), ", ")
        ).ok();
        Ok(())
    }
}
