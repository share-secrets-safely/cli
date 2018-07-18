use base::Vault;
use failure::{err_msg, Error, ResultExt};
use spec::SigningMode;
use std::io::Write;
use std::iter::once;
use std::path::PathBuf;
use util::{export_key, fingerprint_of, new_context, KeyDisplay, KeylistDisplay, UserIdFingerprint};
use TrustModel;

impl Vault {
    pub fn add_recipients(
        &self,
        gpg_key_ids: &[String],
        sign: SigningMode,
        signing_key_id: Option<&str>,
        partitions: &[String],
        output: &mut Write,
    ) -> Result<(), Error> {
        let mut gpg_ctx = new_context()?;
        let partitions: Vec<&Vault> = self.partitions_by_name_or_path(partitions)?;
        let has_multiple_partitions = !self.partitions.is_empty();

        for partition in partitions {
            if let SigningMode::Public = sign {
                let gpg_keys_dir = self.find_gpg_keys_dir().with_context(|_| {
                    "Adding unverified recipients requires you to use a vault that has the `gpg-keys` directory configured"
                })?;
                let imported_gpg_keys_ids = partition.import_keys(&mut gpg_ctx, &gpg_keys_dir, gpg_key_ids, output)?;
                let signing_key = partition
                    .find_signing_key(&mut gpg_ctx, signing_key_id)
                    .with_context(|_| {
                        "Did not manage to find suitable signing key \
                         for re-exporting the recipient keys."
                    })?;
                gpg_ctx.add_signer(&signing_key)?;
                for key_fpr_to_sign in imported_gpg_keys_ids {
                    let key_to_sign = gpg_ctx.get_key(&key_fpr_to_sign)?;
                    gpg_ctx.sign_key(&key_to_sign, None::<&[u8]>, None).with_context(|_| {
                        format_err!(
                            "Could not sign key of recipient {} with signing key {}",
                            key_fpr_to_sign,
                            UserIdFingerprint(&signing_key)
                        )
                    })?;
                    writeln!(
                        output,
                        "Signed recipients key {} with signing key {}",
                        UserIdFingerprint(&key_to_sign),
                        UserIdFingerprint(&signing_key)
                    ).ok();
                }
            }
            let keys = {
                let mut keys_iter = gpg_ctx.find_keys(gpg_key_ids)?;
                let keys: Vec<_> = keys_iter.by_ref().collect::<Result<_, _>>()?;

                if keys_iter.finish()?.is_truncated() {
                    return Err(err_msg("The key list was truncated unexpectedly, while iterating it"));
                }
                keys
            };
            if keys.len() != gpg_key_ids.len() {
                return Err(format_err!(
                    "Found {} viable keys for key-ids ({}), for {} given user ids.",
                    keys.len(),
                    KeylistDisplay(&keys),
                    gpg_key_ids.len()
                ));
            };

            if let Ok(gpg_keys_dir) = self.find_gpg_keys_dir() {
                let mut buf = Vec::new();
                for key in &keys {
                    let (_fingerprint, file_path) = export_key(&mut gpg_ctx, &gpg_keys_dir, key, &mut buf)?;
                    writeln!(
                        output,
                        "Exported public key for user {} to '{}'",
                        KeyDisplay(key),
                        file_path.display()
                    ).ok();
                }
            }

            let mut recipients = partition.recipients_list()?;
            for key in keys {
                recipients.push(fingerprint_of(&key)?);
                writeln!(output, "Added recipient {}", KeyDisplay(&key)).ok();
            }
            partition.write_recipients_list(&mut recipients)?;
            partition.reencrypt(
                &mut gpg_ctx,
                &self.find_trust_model(partition),
                self.gpg_keys_dir_for_auto_import(partition)
                    .as_ref()
                    .map(PathBuf::as_ref),
                has_multiple_partitions,
                output,
            )?;
        }
        Ok(())
    }

    pub fn find_trust_model(&self, partition: &Vault) -> TrustModel {
        partition
            .trust_model
            .as_ref()
            .or_else(|| self.trust_model.as_ref())
            .map(|v| v.to_owned())
            .unwrap_or_else(TrustModel::default)
    }

    pub fn partitions_by_name_or_path(&self, partitions: &[String]) -> Result<Vec<&Vault>, Error> {
        if partitions.is_empty() {
            Ok(vec![self])
        } else {
            let partitions: Vec<usize> = partitions
                .iter()
                .map(|s| Vault::partition_index(s, once(self).chain(self.partitions.iter()), None))
                .collect::<Result<_, _>>()?;
            let mut all_partitions = self.all_in_order();
            all_partitions.retain(|v| partitions.iter().any(|partition_index| v.index == *partition_index));
            Ok(all_partitions)
        }
    }
}
