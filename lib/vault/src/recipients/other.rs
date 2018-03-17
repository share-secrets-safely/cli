use failure::Error;
use std::io::Write;
use base::Vault;
use util::{FingerprintUserId, UserIdFingerprint};
use util::new_context;
use util::extract_at_least_one_secret_key;
use util::export_key;
use std::iter::once;

impl Vault {
    pub fn init_recipients(&self, gpg_key_ids: &[String], output: &mut Write) -> Result<(), Error> {
        let gpg_keys_dir = self.find_gpg_keys_dir()?;
        let mut gpg_ctx = new_context()?;
        let keys = extract_at_least_one_secret_key(&mut gpg_ctx, gpg_key_ids)?;

        let mut buf = Vec::new();
        for key in keys {
            export_key(&mut gpg_ctx, &gpg_keys_dir, &key, &mut buf)?;
            writeln!(output, "Exported public key for {}.", UserIdFingerprint(&key)).ok();
        }
        Ok(())
    }

    pub fn print_recipients(&self, output: &mut Write) -> Result<(), Error> {
        let mut ctx = new_context()?;
        if self.partitions.is_empty() {
            for key in self.recipient_keys(&mut ctx)? {
                writeln!(output, "{}", FingerprintUserId(&key)).ok();
            }
        } else {
            for partition in once(self).chain(self.partitions.iter()) {
                writeln!(output, "{}", partition.url())?;
                for key in partition.recipient_keys(&mut ctx)? {
                    writeln!(output, "{}", FingerprintUserId(&key)).ok();
                }
            }
        }
        Ok(())
    }
}
