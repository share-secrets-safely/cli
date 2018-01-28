use std::io::{self, Write};
use std::path::{Path, PathBuf};

use std::env::{current_dir, set_current_dir};
use std::fs::{self, OpenOptions};
use std::fmt;
use itertools::join;
use failure::{self, err_msg, Error, ResultExt};
use gpgme;

pub fn strip_ext(p: &Path) -> PathBuf {
    let mut p = p.to_owned();
    let stem = p.file_stem().expect(".gpg file extension").to_owned();
    p.set_file_name(stem);
    p
}

pub fn fingerprints_of_keys(keys: &[gpgme::Key]) -> Result<Vec<(&gpgme::Key, String)>, Error> {
    keys.iter()
        .map(|k| fingerprint_of(&k).map(|fpr| (k, fpr)))
        .collect::<Result<Vec<_>, _>>()
        .context("Unexpectedly failed to obtain fingerprint")
        .map_err(Into::into)
}
pub fn write_at(path: &Path) -> io::Result<fs::File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
}

pub struct UserIdFingerprint<'a>(pub &'a gpgme::Key);
impl<'a> fmt::Display for UserIdFingerprint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            join(self.0.user_ids().map(|u| u.id().unwrap_or("[none]")), ", "),
            self.0.fingerprint().unwrap_or("[no fingerprint!]")
        )
    }
}

pub struct FingerprintUserId<'a>(pub &'a gpgme::Key);
impl<'a> fmt::Display for FingerprintUserId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.0.fingerprint().unwrap_or("[no fingerprint!]"),
            join(self.0.user_ids().map(|u| u.id().unwrap_or("[none]")), ", ")
        )
    }
}

pub fn fingerprint_of(key: &gpgme::Key) -> Result<String, failure::Error> {
    key.fingerprint()
        .map_err(|e| {
            e.map(Into::into)
                .unwrap_or_else(|| err_msg("Fingerprint extraction failed"))
        })
        .map(ToOwned::to_owned)
}

pub fn new_context() -> Result<gpgme::Context, gpgme::Error> {
    gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
}

pub struct KeylistDisplay<'a>(pub &'a [gpgme::Key]);

impl<'a> fmt::Display for KeylistDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", join(self.0.iter().map(|k| KeyDisplay(k)), ", "))
    }
}
pub struct KeyDisplay<'a>(pub &'a gpgme::Key);

impl<'a> fmt::Display for KeyDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            join(self.0.user_ids().map(|u| u.id().unwrap_or("[none]")), ", ")
        )
    }
}

pub fn export_key(
    ctx: &mut gpgme::Context,
    gpg_keys_dir: &Path,
    key: &gpgme::Key,
    buf: &mut Vec<u8>,
) -> Result<String, Error> {
    let fingerprint = fingerprint_of(key)?;
    let key_path = gpg_keys_dir.join(&fingerprint);
    ctx.set_armor(true);
    ctx.export_keys(
        [key].iter().map(|k| *k),
        gpgme::ExportMode::empty(),
        &mut *buf,
    ).context(err_msg(
        "Failed to export at least one public key with signatures.",
    ))?;
    write_at(&key_path)
        .and_then(|mut f| f.write_all(buf))
        .context(format!(
            "Could not write public key file at '{}'",
            key_path.display()
        ))?;
    buf.clear();
    Ok(fingerprint.to_owned())
}

pub fn extract_at_least_one_secret_key(
    ctx: &mut gpgme::Context,
    gpg_key_ids: &[String],
) -> Result<Vec<gpgme::Key>, Error> {
    let keys = {
        let mut keys_iter = ctx.find_secret_keys(gpg_key_ids)?;
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
            "No existing GPG key found for which you have a secret key. \
             Please create one with 'gpg --gen-key' and try again.",
        ));
    }

    if keys.len() > 1 && gpg_key_ids.is_empty() {
        return Err(format_err!(
            "Found {} viable keys for key-ids ({}), which is ambiguous. \
             Please specify one with the --gpg-key-id argument.",
            keys.len(),
            KeylistDisplay(&keys)
        ));
    };

    Ok(keys)
}

pub struct ResetCWD {
    cwd: Result<PathBuf, io::Error>,
}
impl ResetCWD {
    pub fn new(next_cwd: &Path) -> Result<Self, Error> {
        let prev_cwd = current_dir();
        set_current_dir(next_cwd).context(format!(
            "Failed to temporarily change the working directory to '{}'",
            next_cwd.display()
        ))?;
        Ok(ResetCWD { cwd: prev_cwd })
    }
}

impl Drop for ResetCWD {
    fn drop(&mut self) {
        self.cwd.as_ref().map(set_current_dir).ok();
    }
}
