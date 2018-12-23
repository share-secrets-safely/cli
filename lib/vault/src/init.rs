use std::path::Path;

use base::Vault;
use failure::{Error, ResultExt};
use spec::WriteMode;
use std::fs::create_dir_all;
use std::io::Write;
use util::export_key_with_progress;
use util::extract_at_least_one_secret_key;
use util::new_context;
use TrustModel;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DirectoryInfo {
    Existed,
    Created,
}

pub fn assure_empty_directory_exists(dir: &Path) -> Result<DirectoryInfo, Error> {
    Ok(if dir.is_dir() {
        let num_entries = dir
            .read_dir()
            .with_context(|_| format!("Failed to open directory '{}' to see if it is empty", dir.display()))?
            .count();
        if num_entries > 0 {
            bail!("Refusing to write into non-empty directory at '{}'", dir.display())
        }
        DirectoryInfo::Existed
    } else {
        create_dir_all(&dir).with_context(|_| format!("Failed to create directory at '{}'", dir.display()))?;
        DirectoryInfo::Created
    })
}

impl Vault {
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
    pub fn init(
        secrets: &Path,
        gpg_key_ids: &[String],
        gpg_keys_dir: &Path,
        recipients_file: &Path,
        vault_path: &Path,
        name: Option<String>,
        trust_model: Option<TrustModel>,
        auto_import: Option<bool>,
        output: &mut Write,
    ) -> Result<Self, Error> {
        let vault = Vault {
            gpg_keys: Some(gpg_keys_dir.to_owned()),
            recipients: recipients_file.to_owned(),
            name,
            secrets: secrets.to_owned(),
            auto_import,
            trust_model,
            ..Default::default()
        }
        .set_resolved_at(vault_path)?;

        let mut gpg_ctx = new_context()?;
        let keys = extract_at_least_one_secret_key(&mut gpg_ctx, gpg_key_ids)?;
        vault.to_file(vault_path, WriteMode::RefuseOverwrite)?;

        let gpg_keys_dir = vault.absolute_path(gpg_keys_dir);
        let recipients_file = vault.absolute_path(recipients_file);
        assure_empty_directory_exists(&gpg_keys_dir).with_context(|_| "Cannot create gpg keys directory")?;

        let mut bytes_buf = Vec::new();
        if recipients_file.is_file() {
            return Err(format_err!(
                "Cannot write recipients into existing file at '{}'",
                recipients_file.display()
            ));
        }

        let mut recipients_fprs = Vec::new();
        for key in keys {
            let (fingerprint, _) = export_key_with_progress(&mut gpg_ctx, &gpg_keys_dir, &key, &mut bytes_buf, output)?;
            recipients_fprs.push(fingerprint);
        }

        vault.write_recipients_list(&mut recipients_fprs)?;
        writeln!(output, "vault initialized at '{}'", vault_path.display()).ok();
        Ok(vault)
    }
}
