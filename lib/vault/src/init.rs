use gpgme;
use util::write_at;

use std::path::Path;

use failure::{err_msg, Error, ResultExt};
use std::fs::create_dir_all;
use std::io::Write;
use types::Vault;

pub fn init(
    gpg_key_ids: &[String],
    gpg_keys_dir: &Path,
    recipients_file: &Path,
    vault_path: &Path,
) -> Result<String, Error> {
    let mut gpg_ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
    let keys = {
        let mut keys_iter = gpg_ctx.find_secret_keys(gpg_key_ids)?;
        let keys: Vec<_> = keys_iter.by_ref().collect::<Result<_, _>>()?;

        if keys_iter.finish()?.is_truncated() {
            return Err(err_msg(
                "The key list was truncated unexpectedly, while iterating it",
            ));
        }
        keys
    };

    if keys.is_empty() {
        return Err(err_msg(
            "No existing GPG key found for which you have a secret key. Please create one and try again.",
        ));
    }

    if keys.len() > 1 && gpg_key_ids.is_empty() {
        return Err(format_err!(
            "Found {} viable keys for key-ids {:?}, which is ambiguous. \
             Please specify one with the --gpg-key-id argument.",
            keys.len(),
            gpg_key_ids
        ));
    };

    let vault = {
        let mut v = Vault::default();
        v.gpg_keys = Some(gpg_keys_dir.to_owned());
        v
    };
    vault.to_file(vault_path)?;

    if gpg_keys_dir.is_dir() {
        let num_entries = gpg_keys_dir
            .read_dir()
            .context(format!(
                "Failed to open directory '{}' to see if it is empty.",
                gpg_keys_dir.display()
            ))?
            .count();
        if num_entries > 0 {
            return Err(format_err!(
                "Cannot export keys into existing, non-empty directory at '{}'",
                gpg_keys_dir.display()
            ));
        }
    } else {
        create_dir_all(gpg_keys_dir).context(format!(
            "Failed to create directory at '{}' for exporting public gpg keys to.",
            gpg_keys_dir.display()
        ))?;
    }

    gpg_ctx.set_armor(true);

    let mut output = Vec::new();
    let mode = gpgme::ExportMode::empty();
    if recipients_file.is_file() {
        return Err(format_err!(
            "Cannot write recipients into existing file at '{}'",
            recipients_file.display()
        ));
    }
    let mut recipients = write_at(recipients_file).context(format!(
        "Failed to open recipients file at '{}'",
        recipients_file.display()
    ))?;
    for key in keys {
        let key_path = {
            let fingerprint = key.fingerprint().map_err(|e| {
                e.map(Into::into)
                    .unwrap_or_else(|| err_msg("Fingerprint extraction failed"))
            })?;
            writeln!(recipients, "{}", fingerprint).context(format!(
                "Could not append fingerprint to file at '{}'",
                recipients_file.display()
            ))?;
            gpg_keys_dir.join(fingerprint)
        };
        gpg_ctx
            .export_keys([key].iter(), mode, &mut output)
            .context(err_msg(
                "Failed to export at least one public key with signatures.",
            ))?;
        write_at(&key_path)
            .and_then(|mut f| f.write_all(&output))
            .context(format!(
                "Could not write public key file at '{}'",
                key_path.display()
            ))?;
        output.clear();
    }
    recipients.flush().context(format!(
        "Failed to flush recipients file at '{}'",
        recipients_file.display()
    ))?;
    Ok(format!("vault initialized at '{}'", vault_path.display()))
}
