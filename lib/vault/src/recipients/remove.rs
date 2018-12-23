use failure::{Error, ResultExt};
use itertools::Itertools;
use std::fs::remove_file;
use std::io::Write;
use std::iter::once;
use std::path::PathBuf;
use util::fingerprints_of_keys;
use util::{new_context, UserIdFingerprint};
use Vault;

impl Vault {
    pub fn remove_recipients(
        &self,
        gpg_key_ids: &[String],
        partitions: &[String],
        output: &mut Write,
    ) -> Result<(), Error> {
        let mut ctx = new_context()?;
        let partitions = self.partitions_by_name_or_path(partitions)?;
        let has_multiple_partitions = !self.partitions.is_empty();
        let gpg_keys_dir_independent_of_auto_import = self.find_gpg_keys_dir().ok();

        for partition in partitions {
            let gpg_keys_dir = self.gpg_keys_dir_for_auto_import(partition);
            let keys_for_ids = partition.keys_by_ids(
                &mut ctx,
                gpg_key_ids,
                "user-id",
                gpg_keys_dir.as_ref().map(PathBuf::as_path),
                output,
            )?;
            let recipients_keys =
                partition.recipient_keys(&mut ctx, gpg_keys_dir.as_ref().map(PathBuf::as_path), output)?;

            let (keys_and_fprs_to_remove, mut remaining_recipients_fprs) = {
                let keys_and_fprs = fingerprints_of_keys(&keys_for_ids)?;
                let recipient_keys_and_fprs = fingerprints_of_keys(&recipients_keys)?;

                let (keys_and_fprs_to_remove, missing) =
                    keys_and_fprs
                        .into_iter()
                        .fold((Vec::new(), Vec::new()), |(mut keys, mut missing), (k, fpr)| {
                            let found = recipient_keys_and_fprs.iter().any(|&(_, ref rkfpr)| *rkfpr == fpr);
                            if found {
                                keys.push((k, fpr));
                            } else {
                                missing.push(k);
                            };
                            (keys, missing)
                        });

                if !missing.is_empty() {
                    return Err(format_err!(
                        "The following key(s) for removal could not be found in the recipients list.\n{}",
                        missing.iter().map(|k| format!("{}", UserIdFingerprint(k))).join("\n")
                    ));
                }
                let recipient_keys_and_fprs = recipient_keys_and_fprs
                    .into_iter()
                    .map(|(_k, fpr)| fpr)
                    .collect::<Vec<_>>();
                (keys_and_fprs_to_remove, recipient_keys_and_fprs)
            };

            for (key, fpr) in keys_and_fprs_to_remove {
                remaining_recipients_fprs.retain(|rfpr| rfpr != &fpr);
                if remaining_recipients_fprs.is_empty() {
                    bail!(
                        "Cannot remove user {} from '{}' as it would be empty afterwards.",
                        UserIdFingerprint(key),
                        partition.recipients_path().display()
                    )
                }

                if let Some(gpg_keys_dir) = gpg_keys_dir_independent_of_auto_import.as_ref() {
                    if self.recipient_used_in_other_partitions(&fpr, partition.index)? {
                        continue;
                    }
                    let fingerprint_path = gpg_keys_dir.join(fpr);
                    if fingerprint_path.is_file() {
                        remove_file(&fingerprint_path)
                            .context(format!("Failed to remove key file at '{}'", fingerprint_path.display(),))?;
                        writeln!(output, "Removed key file at '{}'", fingerprint_path.display())
                    } else {
                        writeln!(
                            output,
                            "Fingerprint key file at '{}' was not existing anymore",
                            fingerprint_path.display()
                        )
                    }
                    .ok();
                }
            }

            let written_file = partition.write_recipients_list(&mut remaining_recipients_fprs)?;
            writeln!(
                output,
                "Wrote changed recipients to file at '{}'",
                written_file.display()
            )
            .ok();

            partition.reencrypt(
                &mut ctx,
                &self.find_trust_model(partition),
                gpg_keys_dir.as_ref().map(PathBuf::as_path),
                has_multiple_partitions,
                output,
            )?;
        }
        Ok(())
    }

    fn recipient_used_in_other_partitions(&self, fpr: &str, index_to_skip: usize) -> Result<bool, Error> {
        for partition in once(self)
            .chain(self.partitions.iter())
            .filter(|p| p.index != index_to_skip)
        {
            if partition
                .recipients_list()?
                .iter()
                .any(|rfpr| rfpr == fpr || fpr.starts_with(rfpr))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
