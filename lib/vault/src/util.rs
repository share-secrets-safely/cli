use std::io;
use std::path::Path;
use std::fs::{self, OpenOptions};
use std::fmt;
use itertools::join;
use failure::{self, err_msg};
use gpgme;

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
