use failure::{Error, ResultExt};
use std::io::Write;
use Vault;
use util::{new_context, UserIdFingerprint};
use itertools::Itertools;
use std::fs::remove_file;
use util::fingerprints_of_keys;

impl Vault {
    pub fn remove_recipients(&self, gpg_key_ids: &[String], output: &mut Write) -> Result<(), Error> {
        let mut ctx = new_context()?;
        let keys_for_ids = self.keys_by_ids(&mut ctx, gpg_key_ids, "user-id")?;
        let recipients_keys = self.recipient_keys(&mut ctx)?;

        let (keys_and_fprs_to_remove, mut remaining_recipients_fprs) = {
            let keys_and_fprs = fingerprints_of_keys(&keys_for_ids)?;
            let recipient_keys_and_fprs = fingerprints_of_keys(&recipients_keys)?;

            let (keys_and_fprs_to_remove, missing) = keys_and_fprs.into_iter().fold(
                (Vec::new(), Vec::new()),
                |(mut keys, mut missing), (k, fpr)| {
                    let found = recipient_keys_and_fprs
                        .iter()
                        .any(|&(_, ref rkfpr)| *rkfpr == fpr);
                    if found {
                        keys.push((k, fpr));
                    } else {
                        missing.push(k);
                    };
                    (keys, missing)
                },
            );

            if !missing.is_empty() {
                return Err(format_err!(
                    "The following key(s) for removal could not be found in the recipients list.\n{}",
                    missing
                        .iter()
                        .map(|k| format!("{}", UserIdFingerprint(k)))
                        .join("\n")
                ));
            }
            let recipient_keys_and_fprs = recipient_keys_and_fprs
                .into_iter()
                .map(|(_k, fpr)| fpr)
                .collect::<Vec<_>>();
            (keys_and_fprs_to_remove, recipient_keys_and_fprs)
        };

        let gpg_keys_dir = self.gpg_keys_dir();

        for (_key, fpr) in keys_and_fprs_to_remove {
            remaining_recipients_fprs.retain(|rfpr| rfpr != &fpr);

            if let Ok(gpg_keys_dir) = gpg_keys_dir.as_ref() {
                let fingerprint_path = gpg_keys_dir.join(fpr);
                if fingerprint_path.is_file() {
                    remove_file(&fingerprint_path).context(format!(
                        "Failed to remove key file at '{}'",
                        fingerprint_path.display(),
                    ))?;
                    writeln!(
                        output,
                        "Removed key file at '{}'",
                        fingerprint_path.display()
                    )
                } else {
                    writeln!(
                        output,
                        "Fingerprint key file at '{}' was not existing anymore",
                        fingerprint_path.display()
                    )
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
