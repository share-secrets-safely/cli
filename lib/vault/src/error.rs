use serde_yaml;
use std::io;
use std::path::{Path, PathBuf};
use std::fmt;
use failure::Fail;
use gpgme;
use failure;
use mktemp::Temp;
use util::write_at;
use std::fs::File;
use util::FingerprintUserId;

#[derive(Debug, Fail)]
#[fail(display = "The content was not encrypted for you.")]
pub struct DecryptionError {
    #[cause]
    pub cause: gpgme::Error,
}

impl DecryptionError {
    pub fn caused_by(err: gpgme::Error, alternative_text: &'static str) -> failure::Error {
        if err.code() == gpgme::Error::NO_SECKEY.code() {
            failure::Error::from(DecryptionError { cause: err })
        } else {
            err.context(alternative_text).into()
        }
    }
}

#[derive(Debug, Fail)]
pub struct EncryptionError {
    pub msg: String,
    pub offending_recipients: Vec<String>,
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)?;
        for info in &self.offending_recipients {
            write!(f, "\n{}", info)?;
        }
        Ok(())
    }
}

fn find_offending_keys(ctx: &mut gpgme::Context, keys: &[gpgme::Key]) -> Result<Vec<String>, failure::Error> {
    let mut output = Vec::new();
    let mut obuf = Vec::<u8>::new();
    let temp = Temp::new_file()?;
    let temp_path = temp.to_path_buf();
    {
        let _ibuf = write_at(&temp_path)?;
    }
    let mut ibuf = File::open(&temp_path)?;
    for key in keys {
        if let Err(err) = ctx.encrypt(Some(key), &mut ibuf, &mut obuf) {
            output.push(format!(
                "Could not encrypt for recipient {} with error: {}",
                FingerprintUserId(key),
                err
            ));
        }
    }
    Ok(output)
}

impl EncryptionError {
    pub fn caused_by(
        err: gpgme::Error,
        alternative_text: String,
        ctx: &mut gpgme::Context,
        keys: &[gpgme::Key],
    ) -> failure::Error {
        failure::Error::from(EncryptionError {
            msg: if err.code() == gpgme::Error::UNUSABLE_PUBKEY.code() {
                "At least one recipient you try to encrypt for is untrusted. \
                 Consider (locally) signing the key with `gpg --sign-key <recipient>` \
                 or ultimately trusting them."
                    .into()
            } else {
                alternative_text
            },
            offending_recipients: match find_offending_keys(ctx, keys) {
                Ok(v) => v,
                Err(e) => return e,
            },
        })
    }
}

#[derive(Debug, Fail)]
pub enum VaultError {
    ConfigurationFileExists(PathBuf),
    PartitionUnsupported,
    ReadFile {
        #[cause]
        cause: io::Error,
        path: PathBuf,
    },
    WriteFile {
        #[cause]
        cause: io::Error,
        path: PathBuf,
    },
    Deserialization {
        #[cause]
        cause: serde_yaml::Error,
        path: PathBuf,
    },
    Serialization {
        #[cause]
        cause: serde_yaml::Error,
        path: PathBuf,
    },
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::VaultError::*;
        match *self {
            PartitionUnsupported => f.write_str("Cannot perform this operation on a partition"),
            ConfigurationFileExists(ref path) => writeln!(
                f,
                "Cannot overwrite vault configuration file as it already exists at '{}'",
                path.display()
            ),
            Serialization { ref path, .. } => writeln!(
                f,
                "Failed to serialize vault configuration file at '{}'",
                path.display()
            ),
            Deserialization { ref path, .. } => writeln!(
                f,
                "Failed to deserialize vault configuration file at '{}'",
                path.display()
            ),
            WriteFile { ref path, .. } => writeln!(
                f,
                "Failed to write vault configuration file at '{}'",
                path.display()
            ),
            ReadFile { ref path, .. } => writeln!(
                f,
                "Failed to read vault configuration file at '{}'",
                path.display()
            ),
        }
    }
}

pub enum IOMode {
    Read,
    Write,
}

impl VaultError {
    pub fn from_io_err(cause: io::Error, path: &Path, mode: &IOMode) -> Self {
        match *mode {
            IOMode::Write => VaultError::WriteFile {
                cause,
                path: path.to_owned(),
            },
            IOMode::Read => VaultError::ReadFile {
                cause,
                path: path.to_owned(),
            },
        }
    }
}

pub trait FailExt: Fail {
    fn first_cause_of<T: Fail>(&self) -> Option<&T>;
}

impl<F> FailExt for F
where
    F: Fail,
{
    fn first_cause_of<T: Fail>(&self) -> Option<&T> {
        self.causes().filter_map(|c| c.downcast_ref::<T>()).next()
    }
}

// https://github.com/withoutboats/failure/pull/124
pub fn first_cause_of_type<T: Fail>(root: &failure::Error) -> Option<&T> {
    root.causes().filter_map(|c| c.downcast_ref::<T>()).next()
}
