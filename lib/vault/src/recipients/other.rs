use std::path::PathBuf;

use failure::{Error, ResultExt};
use std::fs::File;
use std::io::{BufReader, Write, BufRead};
use vault::Vault;
use util::{FingerprintUserId, UserIdFingerprint};
use util::new_context;
use util::extract_at_least_one_secret_key;
use util::export_key;

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

    pub fn gpg_keys_dir(&self) -> Result<PathBuf, Error> {
        let unknown_path = PathBuf::from("<unknown>");
        self.gpg_keys
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
            })
    }

    pub fn init_recipients(&self, gpg_key_ids: &[String], output: &mut Write) -> Result<(), Error> {
        let gpg_keys_dir = self.gpg_keys_dir()?;
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
}
