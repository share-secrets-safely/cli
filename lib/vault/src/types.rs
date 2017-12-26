use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, Read, Write};
use serde_yaml;
use util::write_at;
use error::{IOMode, VaultError};
use failure::{Error, ResultExt};

pub fn recipients_default() -> PathBuf {
    PathBuf::from(".gpg-id")
}

pub fn at_default() -> PathBuf {
    PathBuf::from(".")
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Vault {
    #[serde(default = "at_default")] pub at: PathBuf,
    pub gpg_keys: Option<PathBuf>,
    #[serde(default = "recipients_default")] pub recipients: PathBuf,
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
                .map_err(|cause| VaultError::from_io_err(cause, path, &IOMode::Read))?)
        };
        serde_yaml::from_reader(reader).map_err(|cause| VaultError::Deserialization {
            cause,
            path: path.to_owned(),
        })
    }

    pub fn to_file(&self, path: &Path) -> Result<(), VaultError> {
        if path.exists() {
            return Err(VaultError::ConfigurationFileExists(path.to_owned()));
        }
        let mut file =
            write_at(path).map_err(|cause| VaultError::from_io_err(cause, path, &IOMode::Write))?;
        serde_yaml::to_writer(&file, self)
            .map_err(|cause| VaultError::Serialization {
                cause,
                path: path.to_owned(),
            })
            .and_then(|_| {
                writeln!(file).map_err(|cause| VaultError::from_io_err(cause, path, &IOMode::Write))
            })
    }

    fn absolute_path(&self, path: &Path) -> PathBuf {
        self.at.join(path)
    }

    pub fn recipients(&self) -> Result<Vec<String>, Error> {
        let recipients_file_path = self.absolute_path(&self.recipients);
        let rfile = File::open(&recipients_file_path)
            .map(BufReader::new)
            .context(format!(
                "Could not open recipients file at '{}' for reading",
                recipients_file_path.display()
            ))?;
        Ok(rfile.lines().collect::<Result<_, _>>().context(format!(
            "Could not read all recipients from file at '{}'",
            recipients_file_path.display()
        ))?)
    }
}
