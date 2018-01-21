use failure::Error;
use std::io::Write;
use Vault;
use util::{UserIdFingerprint, new_context};
use itertools::Itertools;
use util::fingerprint_of;
use std::fs::remove_file;

impl Vault {
    pub fn remove_recipients(&self, gpg_key_ids: &[String], output: &mut Write) -> Result<(), Error> {
        let mut ctx = new_context()?;
        let keys_for_ids = self.keys_by_ids(&mut ctx, gpg_key_ids, "user-id")?;
        let recipients_keys = self.recipient_keys(&mut ctx)?;

        // TODO: use fingerprint_of but make failure to get it fatal
        let keys_to_remove = {
            let mut missing = Vec::new();
            let keys_for_ids = keys_for_ids
                .iter()
                .filter_map(|k| k.fingerprint().ok().map(|fpr| (k, fpr)))
                .filter_map(|(k, fpr)| {
                    let found = recipients_keys
                        .iter()
                        .filter_map(|k| k.fingerprint().ok())
                        .any(|rkfpr| rkfpr == fpr);
                    if found {
                        Some(k)
                    } else {
                        missing.push(k);
                        None
                    }
                })
                .collect::<Vec<_>>();

            if !missing.is_empty() {
                return Err(format_err!(
                    "The following key(s) for removal could not be found in the recipients list.\n{}",
                    missing
                        .iter()
                        .map(|k| format!("{}", UserIdFingerprint(k)))
                        .join("\n")
                ));
            }

            keys_for_ids
        };

        let mut remaining_recipients_fprs = recipients_keys
            .iter()
            .map(|k| fingerprint_of(&k))
            .collect::<Result<Vec<_>, _>>()?;
        let gpg_keys_dir = self.gpg_keys_dir();

        for key in keys_to_remove {
            let fpr = fingerprint_of(key)?;
            remaining_recipients_fprs.retain(|rfpr| rfpr != &fpr);

            if let Ok(gpg_keys_dir) = gpg_keys_dir.as_ref() {
                let fingerprint_path = gpg_keys_dir.join(fpr);
                match remove_file(&fingerprint_path) {
                    Ok(_) => {
                        writeln!(
                            output,
                            "Removed key file at '{}'",
                            fingerprint_path.display()
                        )
                    }
                    Err(e) => {
                        writeln!(
                            output,
                            "Failed to remove key file at '{}' with error: {}",
                            fingerprint_path.display(),
                            e
                        )
                    }
                }.ok();
            }
        }

        let written_file = self.write_recipients_list(&mut remaining_recipients_fprs)?;
        writeln!(
            output,
            "Wrote changed recipients to file at '{}'",
            written_file.display()
        ).ok();

        self.reencrypt(&mut ctx, output)?;
        Ok(())
    }
}
