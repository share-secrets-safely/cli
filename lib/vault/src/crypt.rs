extern crate s3_types;

use s3_types::VaultSpec;
use vault::Vault;
use failure::{err_msg, Error, ResultExt};
use util::UserIdFingerprint;
use std::io::Write;
use itertools::join;
use gpgme;
use std::path::Path;
use std::fs::File;
use s3_types::gpg_output_filename;

impl Vault {
    pub fn decrypt(&self, path: &Path, w: &mut Write) -> Result<(), Error> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let gpg_path = gpg_output_filename(path)?;
        let mut input = File::open(&gpg_path)
            .or_else(|_| File::open(path))
            .context(format!(
                "Could not open input file at '{}' for reading. Tried '{}' as well.",
                gpg_path.display(),
                path.display()
            ))?;
        let mut output = Vec::new();
        ctx.decrypt(&mut input, &mut output)
            .context("Failed to decrypt data")?;

        w.write_all(&output)
            .context("Could not write out all decrypted data.")?;
        Ok(())
    }

    pub fn encrypt(&self, specs: &[VaultSpec]) -> Result<String, Error> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let recipients = self.recipients_list()?;
        if recipients.is_empty() {
            return Err(format_err!(
                "No recipients found in recipients file at '{}'",
                self.recipients.display()
            ));
        }
        let keys: Vec<gpgme::Key> = ctx.find_keys(&recipients)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|k| k.can_encrypt())
            .collect();
        if keys.len() != recipients.len() {
            let diff = recipients.len() - keys.len();
            let mut msg = vec![
                if diff > 0 {
                    format!(
                        "Didn't find a key for {} recipients in the gpg database.",
                        diff
                    )
                } else {
                    format!(
                        "Found {} additional keys to encrypt for, which is unexpected.",
                        diff
                    )
                },
            ];
            msg.push("All recipients:".into());
            msg.push(recipients.join(", "));
            msg.push("All recipients found in gpg database".into());
            msg.extend(keys.iter().map(|k| format!("{}", UserIdFingerprint(k))));
            return Err(err_msg(msg.join("\n")));
        }

        let mut encrypted = Vec::new();
        for spec in specs {
            let input = {
                let mut buf = Vec::new();
                spec.open_input()?.read_to_end(&mut buf).context(format!(
                    "Could not read all input from '{}' into buffer",
                    spec.source()
                        .map(|s| format!("{}", s.display()))
                        .unwrap_or_else(|| "<stdin>".into())
                ))?;
                buf
            };
            let mut output = Vec::new();
            ctx.encrypt(&keys, input, &mut output)
                .context(format!("Failed to encrypt {}", spec))?;
            spec.open_output(&self.resolved_at)?
                .write_all(&output)
                .context(format!(
                    "Failed to write all encrypted data to '{}'",
                    spec.destination().display(),
                ))?;
            encrypted.push(spec.destination());
        }
        Ok(format!(
            "Added {}.",
            join(encrypted.iter().map(|p| format!("'{}'", p.display())), ", ")
        ))
    }
}
