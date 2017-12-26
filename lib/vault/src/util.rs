use std::io;
use std::path::Path;
use std::fs::{self, OpenOptions};
use std::fmt;
use itertools::join;
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
            self.0.fingerprint().unwrap_or("[no fingerprint!]"),
            join(self.0.user_ids().map(|u| u.id().unwrap_or("[none]")), ", ")
        )
    }
}
