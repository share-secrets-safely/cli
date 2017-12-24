use serde_yaml;
use std::io;
use std::path::{Path, PathBuf};
use std::fmt;

#[derive(Debug, Fail)]
pub enum VaultError {
    ConfigurationFileExists(PathBuf),
    ReadFile {
        #[cause] cause: io::Error,
        path: PathBuf,
    },
    WriteFile {
        #[cause] cause: io::Error,
        path: PathBuf,
    },
    Deserialization {
        #[cause] cause: serde_yaml::Error,
        path: PathBuf,
    },
    Serialization {
        #[cause] cause: serde_yaml::Error,
        path: PathBuf,
    },
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::VaultError::*;
        match *self {
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
                "Failed to create vault configuration file at '{}'",
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
