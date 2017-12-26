use s3_types::VaultSpec;
use types::Vault;
use failure::{err_msg, Error, ResultExt};
use util::UserIdFingerprint;
use std::io::Write;
use itertools::join;
use gpgme;

pub fn add(vault: Vault, specs: &[VaultSpec]) -> Result<String, Error> {
    let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
    let recipients = vault.recipients()?;
    if recipients.is_empty() {
        return Err(format_err!(
            "No recipients found in recipients file at '{}'",
            vault.recipients.display()
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
        msg.extend(keys.iter().map(|k| format!("{}", UserIdFingerprint(&k))));
        return Err(err_msg(msg.join("\n")));
    }

    for spec in specs {
        let input = {
            let mut buf = Vec::new();
            spec.open_input()?.read_to_end(&mut buf).context(format!(
                "Could not read all input from '{}' into buffer",
                spec.source()
                    .map(|s| format!("{}", s.display()))
                    .unwrap_or("<stdin>".into())
            ))?;
            buf
        };
        let mut output = Vec::new();
        ctx.encrypt(&keys, input, &mut output)
            .context(format!("Failed to encrypt {}", spec))?;
        spec.open_output(&vault.at)?
            .write_all(&output)
            .context(format!(
                "Failed to write all encrypted data to '{}'",
                spec.destination().display(),
            ))?;
    }
    Ok(format!(
        "Successfully added [{}] to the vault",
        join(specs, ", ")
    ))
}
