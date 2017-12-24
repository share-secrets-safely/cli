use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{stdin, Read, Write};
use serde_yaml;
use util::write_at;
use error::{IOMode, VaultError};

pub fn recipients_default() -> String {
    String::from(".gpg-id")
}

pub fn at_default() -> String {
    String::from(".")
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Vault {
    #[serde(default = "at_default")] pub at: String,
    pub gpg_keys: Option<PathBuf>,
    #[serde(default = "recipients_default")] pub recipients: String,
}

impl Default for Vault {
    fn default() -> Self {
        Vault {
            at: at_default(),
            gpg_keys: None,
            recipients: recipients_default(),
        }
    }
}

impl Vault {
    pub fn from_file(path: &Path) -> Result<Vault, VaultError> {
        let reader: Box<Read> = if path == Path::new("-") {
            Box::new(stdin())
        } else {
            Box::new(File::open(path)
                .map_err(|cause| VaultError::from_io_err(cause, path, IOMode::Read))?)
        };
        serde_yaml::from_reader(reader).map_err(|cause| VaultError::Deserialization {
            cause,
            path: path.to_owned(),
        })
    }

    pub fn to_file(&self, path: &Path) -> Result<(), VaultError> {
        let mut file =
            write_at(path).map_err(|cause| VaultError::from_io_err(cause, path, IOMode::Write))?;
        serde_yaml::to_writer(&file, self)
            .map_err(|cause| VaultError::Serialization {
                cause,
                path: path.to_owned(),
            })
            .and_then(|_| {
                writeln!(file).map_err(|cause| VaultError::from_io_err(cause, path, IOMode::Write))
            })
    }
}
