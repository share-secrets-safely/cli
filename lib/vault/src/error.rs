use serde_yaml;
use std::io;

#[derive(Debug, Fail)]
#[fail(display = "The top-level error related to handling the vault.")]
pub enum ExportKeysError {
    #[fail(display = "Could not create directory at '{}'", path)]
    CreateDirectory {
        #[cause] cause: io::Error,
        path: String,
    },
}

#[derive(Debug, Fail)]
#[fail(display = "The top-level error related to handling the vault.")]
pub enum VaultError {
    #[fail(display = "Could not access vault configuration file for reading '{}'", path)]
    ReadFile {
        #[cause] cause: io::Error,
        path: String,
    },
    #[fail(display = "Could not open vault configuration file for writing '{}'", path)]
    WriteFile {
        #[cause] cause: io::Error,
        path: String,
    },
    #[fail(display = "Could not deserialize vault configuration file at '{}'", path)]
    Deserialization {
        #[cause] cause: serde_yaml::Error,
        path: String,
    },
    #[fail(display = "Could not serialize vault configuration file to '{}'", path)]
    Serialization {
        #[cause] cause: serde_yaml::Error,
        path: String,
    },
}

pub enum IOMode {
    Read,
    Write,
}

impl VaultError {
    pub fn from_io_err(cause: io::Error, path: &str, mode: IOMode) -> Self {
        match mode {
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
