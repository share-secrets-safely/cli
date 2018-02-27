use std::path::Path;

use failure::{Error, ResultExt};
use std::fs::create_dir_all;
use base::Vault;
use util::new_context;
use util::extract_at_least_one_secret_key;
use util::export_key;
use sheesy_types::WriteMode;

impl Vault {
    pub fn init(
        secrets: &Path,
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
            secrets: secrets.to_owned(),
            ..Default::default()
        }.set_resolved_at(vault_path)?;

        let mut gpg_ctx = new_context()?;
        let keys = extract_at_least_one_secret_key(&mut gpg_ctx, gpg_key_ids)?;
        vault.to_file(vault_path, WriteMode::RefuseOverwrite)?;

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

        let mut recipients_fprs = Vec::new();
        for key in keys {
            let fingerprint = export_key(&mut gpg_ctx, &gpg_keys_dir, &key, &mut output)?;
            recipients_fprs.push(fingerprint);
        }

        vault.write_recipients_list(&mut recipients_fprs)?;
        Ok(vault)
    }
}
