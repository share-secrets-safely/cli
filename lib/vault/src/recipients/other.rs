use failure::Error;
use std::io::Write;
use base::Vault;
use util::{FingerprintUserId, UserIdFingerprint};
use util::new_context;
use util::extract_at_least_one_secret_key;
use util::export_key;

impl Vault {
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
