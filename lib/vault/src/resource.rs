use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fs::{remove_file, File};
use std::mem;

use mktemp::Temp;
use itertools::join;
use gpgme;
use base::Vault;
use failure::{Error, ResultExt};
use error::FailExt;
use spec::{gpg_output_filename, SpecSourceType, VaultSpec};
use spec::{CreateMode, Destination, WriteMode};
use error::{DecryptionError, EncryptionError};
use util::{new_context, strip_ext, write_at};
use util::run_editor;
use std::iter::once;
use TrustModel;
use util::flags_for_model;

fn encrypt_buffer(
    ctx: &mut gpgme::Context,
    input: &[u8],
    keys: &[gpgme::Key],
    model: &TrustModel,
) -> Result<Vec<u8>, Error> {
    let mut encrypted_bytes = Vec::<u8>::new();
    let flags = flags_for_model(model);
    ctx.encrypt_with_flags(keys, input, &mut encrypted_bytes, flags)
        .map_err(|e: gpgme::Error| EncryptionError::caused_by(e, "Failed to encrypt data.".into(), ctx, keys))?;
    Ok(encrypted_bytes)
}

impl Vault {
    pub fn edit(
        &self,
        path: &Path,
        editor: &Path,
        mode: &CreateMode,
        try_encrypt: bool,
        output: &mut Write,
    ) -> Result<(), Error> {
        let file = Temp::new_file().context("Could not create temporary file to decrypt to.")?;
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
        if try_encrypt {
            let (partition, _) = self.partition_by_owned_path(tempfile_path.clone())?;
            self.encrypt_buffer(
                b"",
                self.gpg_keys_dir_for_auto_import(partition)
                    .as_ref()
                    .map(PathBuf::as_path),
            ).context("Aborted edit operation as you cannot encrypt resources.")?;
        }
        run_editor(editor.as_os_str(), &tempfile_path)?;
        let mut zero = Vec::new();
        self.encrypt(
            &[
                VaultSpec {
                    src: SpecSourceType::Path(tempfile_path.clone()),
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
        let (partition, path) = self.partition_by_owned_path(path.to_owned())?;
        let resolved_absolute_path = partition.secrets_path().join(path);
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

        w.write_all(&output).context("Could not write out all decrypted data.")?;
        Ok(path_for_decryption)
    }

    pub fn remove(&self, specs: &[PathBuf], output: &mut Write) -> Result<(), Error> {
        for path_to_remove in specs {
            let (partition, path_to_remove) = self.partition_by_owned_path(path_to_remove.to_owned())?;
            let path = {
                let spec = VaultSpec {
                    src: SpecSourceType::Stdin,
                    dst: path_to_remove,
                };
                let gpg_path = spec.output_in(&partition.secrets_path(), Destination::ReolveAndAppendGpg)?;
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
            remove_file(&path).context(format!("Failed to remove file at '{}'.", path.display()))?;
            writeln!(output, "Removed file at '{}'", path.display()).ok();
        }
        Ok(())
    }

    pub fn encrypt_buffer(&self, input: &[u8], gpg_keys_dir: Option<&Path>) -> Result<Vec<u8>, Error> {
        let mut ctx = new_context()?;
        let keys = self.recipient_keys(&mut ctx, gpg_keys_dir)?;

        let encrypted_bytes = encrypt_buffer(
            &mut ctx,
            input,
            &keys,
            &self.trust_model.clone().unwrap_or_else(TrustModel::default),
        )?;
        Ok(encrypted_bytes)
    }

    pub fn partition_by_owned_path(&self, path: PathBuf) -> Result<(&Vault, PathBuf), Error> {
        if self.partitions.is_empty() {
            Ok((self, path))
        } else {
            let partition = once(self)
                .chain(&self.partitions)
                .find(|p| path.starts_with(&p.secrets))
                .ok_or_else(|| {
                    format_err!("Path '{}' could not be associated with any partition. Prefix it with the partition resource directory.", path.display())
                })?;
            Ok((
                partition,
                path.strip_prefix(&partition.secrets)
                    .expect("success if 'starts_with' succeeds")
                    .to_owned(),
            ))
        }
    }

    pub fn partition_by_owned_spec(&self, mut spec: VaultSpec) -> Result<(&Vault, VaultSpec), Error> {
        let (partition, path) = self.partition_by_owned_path(spec.dst)?;
        spec.dst = path;
        Ok((partition, spec))
    }

    pub fn partition_by_spec(&self, spec: &VaultSpec) -> Result<(&Vault, VaultSpec), Error> {
        self.partition_by_owned_spec(spec.clone())
    }

    pub fn encrypt(
        &self,
        specs: &[VaultSpec],
        mode: WriteMode,
        dst_mode: Destination,
        output: &mut Write,
    ) -> Result<(), Error> {
        let mut ctx = new_context()?;
        let mut lut: Vec<Option<(PathBuf, Vec<gpgme::Key>)>> = vec![None; 1 + self.partitions.len()];
        let mut encrypted_destinations = Vec::new();

        for spec in specs {
            {
                let (partition, spec) = self.partition_by_spec(spec)?;
                let (secrets_dir, keys) = match &mut lut[partition.index] {
                    &mut Some((ref secrets_dir, ref keys)) => (secrets_dir, keys),
                    none => {
                        let gpg_keys_dir = self.gpg_keys_dir_for_auto_import(partition);
                        mem::replace(
                            none,
                            Some((
                                partition.secrets_path(),
                                partition.recipient_keys(&mut ctx, gpg_keys_dir.as_ref().map(PathBuf::as_path))?,
                            )),
                        );
                        let some = none;
                        let &(ref secrets_dir, ref keys) = some.as_ref().expect("the content that was just put in");
                        (secrets_dir, keys)
                    }
                };
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
                let mut encrypted_bytes = encrypt_buffer(&mut ctx, &input, keys, &self.find_trust_model(partition))?;
                spec.open_output_in(secrets_dir, mode, dst_mode, output)?
                    .write_all(&encrypted_bytes)
                    .context(format!(
                        "Failed to write all encrypted data to '{}'.",
                        spec.destination().display(),
                    ))?;
            }
            encrypted_destinations.push(spec.destination());
        }
        writeln!(
            output,
            "Added {}.",
            join(
                encrypted_destinations.iter().map(|p| format!("'{}'", p.display())),
                ", "
            )
        ).ok();
        Ok(())
    }
}
