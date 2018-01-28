use failure::{err_msg, Error, ResultExt};
use std::io::Write;
use base::Vault;
use util::{export_key, fingerprint_of, new_context, KeyDisplay, KeylistDisplay, UserIdFingerprint};
use sheesy_types::SigningMode;

impl Vault {
    pub fn add_recipients(
        &self,
        gpg_key_ids: &[String],
        sign: SigningMode,
        signing_key_id: Option<&str>,
        output: &mut Write,
    ) -> Result<(), Error> {
        let mut gpg_ctx = new_context()?;
        if let SigningMode::Public = sign {
            let gpg_keys_dir = self.gpg_keys_dir().context(
                "Adding unverified recipients requires you to use a vault that has the `gpg-keys` directory configured",
            )?;
            let imported_gpg_keys_ids = self.import_keys(&mut gpg_ctx, &gpg_keys_dir, gpg_key_ids, output)?;
            let signing_key = self.find_signing_key(&mut gpg_ctx, signing_key_id)
                .context(
                    "Did not manage to find suitable signing key \
                     for re-exporting the recipient keys.",
                )?;
            gpg_ctx.add_signer(&signing_key)?;
            for key_fpr_to_sign in imported_gpg_keys_ids {
                let key_to_sign = gpg_ctx.find_key(&key_fpr_to_sign)?;
                gpg_ctx
                    .sign_key(&key_to_sign, None::<&[u8]>, None)
                    .context(format_err!(
                        "Could not sign key of recipient {} with signing key {}",
                        key_fpr_to_sign,
                        UserIdFingerprint(&signing_key)
                    ))?;
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
                return Err(err_msg(
                    "The key list was truncated unexpectedly, while iterating it",
                ));
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

        if let Some(gpg_keys_dir) = self.gpg_keys.as_ref() {
            let gpg_keys_dir = self.absolute_path(gpg_keys_dir);
            let mut buf = Vec::new();
            for key in &keys {
                let fingerprint = export_key(&mut gpg_ctx, &gpg_keys_dir, key, &mut buf)?;
                writeln!(
                    output,
                    "Exported key '{}' for user {}",
                    fingerprint,
                    KeyDisplay(key)
                ).ok();
            }
        }

        let mut recipients = self.recipients_list()?;
        for key in keys {
            recipients.push(fingerprint_of(&key)?);
            writeln!(output, "Added recipient {}", KeyDisplay(&key)).ok();
        }
        self.write_recipients_list(&mut recipients)?;
        self.reencrypt(&mut gpg_ctx, output)?;
        Ok(())
    }
}
