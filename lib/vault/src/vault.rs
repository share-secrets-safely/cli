use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, stdin, Read, Write};
use std::env::{current_dir, set_current_dir};
use serde_yaml;
use util::write_at;
use error::{IOMode, VaultError};
use failure::{Error, ResultExt};
use glob::glob;

pub const GPG_GLOB: &str = "**/*.gpg";
pub fn recipients_default() -> PathBuf {
    PathBuf::from(".gpg-id")
}

pub fn secrets_default() -> PathBuf {
    PathBuf::from(".")
}

pub fn strip_ext(p: &Path) -> PathBuf {
    let mut p = p.to_owned();
    let stem = p.file_stem().expect(".gpg file extension").to_owned();
    p.set_file_name(stem);
    p
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
#[derive(Deserialize, PartialEq, Serialize, Debug, Clone)]
pub struct Vault {
    pub name: Option<String>,
    #[serde(skip)]
    pub resolved_at: PathBuf,
    #[serde(skip)]
    pub vault_path: Option<PathBuf>,
    #[serde(default = "secrets_default")]
    pub secrets: PathBuf,
    pub gpg_keys: Option<PathBuf>,
    #[serde(default = "recipients_default")]
    pub recipients: PathBuf,
}

impl Default for Vault {
    fn default() -> Self {
        Vault {
            vault_path: None,
            name: None,
            secrets: secrets_default(),
            resolved_at: secrets_default(),
            gpg_keys: None,
            recipients: recipients_default(),
        }
    }
}

fn split_documents<R: Read>(mut r: R) -> Result<Vec<String>, Error> {
    use yaml_rust::{YamlEmitter, YamlLoader};

    let mut buf = String::new();
    r.read_to_string(&mut buf)?;

    let docs = YamlLoader::load_from_str(&buf).context(
        "YAML deserialization failed",
    )?;
    Ok(
        docs.iter()
            .map(|d| {
                let mut out_str = String::new();
                {
                    let mut emitter = YamlEmitter::new(&mut out_str);
                    emitter.dump(d).expect(
                        "dumping a valid yaml into a string to work",
                    );
                }
                out_str
            })
            .collect(),
    )
}

impl Vault {
    pub fn from_file(path: &Path) -> Result<Vec<Vault>, Error> {
        let path_is_stdin = path == Path::new("-");
        let reader: Box<Read> = if path_is_stdin {
            Box::new(stdin())
        } else {
            Box::new(File::open(path).map_err(|cause| {
                VaultError::from_io_err(cause, path, &IOMode::Read)
            })?)
        };
        Ok(split_documents(reader)?
            .iter()
            .map(|s| {
                serde_yaml::from_str(s)
                    .map_err(|cause| {
                        VaultError::Deserialization {
                            cause,
                            path: path.to_owned(),
                        }
                    })
                    .map_err(Into::into)
                    .and_then(|v: Vault| v.set_resolved_at(path))
            })
            .collect::<Result<_, _>>()?)
    }

    pub fn set_resolved_at(mut self, vault_file: &Path) -> Result<Self, Error> {
        self.resolved_at = normalize(&vault_file.parent().ok_or_else(|| {
            format_err!("The vault file path '{}' is invalid.", vault_file.display())
        })?);
        self.vault_path = Some(vault_file.to_owned());
        Ok(self)
    }

    pub fn to_file(&self, path: &Path) -> Result<(), VaultError> {
        if path.exists() {
            return Err(VaultError::ConfigurationFileExists(path.to_owned()));
        }
        let mut file = write_at(path).map_err(|cause| {
            VaultError::from_io_err(cause, path, &IOMode::Write)
        })?;
        serde_yaml::to_writer(&file, self)
            .map_err(|cause| {
                VaultError::Serialization {
                    cause,
                    path: path.to_owned(),
                }
            })
            .and_then(|_| {
                writeln!(file).map_err(|cause| VaultError::from_io_err(cause, path, &IOMode::Write))
            })
    }

    pub fn absolute_path(&self, path: &Path) -> PathBuf {
        normalize(&self.resolved_at.join(path))
    }

    pub fn secrets_path(&self) -> PathBuf {
        normalize(&self.absolute_path(&self.secrets))
    }
    pub fn url(&self) -> String {
        format!(
            "syv://{}{}",
            self.name
                .as_ref()
                .map(|s| format!("{}@", s))
                .unwrap_or_else(String::new),
            self.secrets_path().display()
        )
    }

    pub fn list(&self, w: &mut Write) -> Result<(), Error> {
        writeln!(w, "{}", self.url())?;
        let _change_cwd = ResetCWD::new(&self.secrets_path())?;
        for entry in glob(GPG_GLOB).expect("valid pattern").filter_map(
            Result::ok,
        )
        {
            writeln!(w, "{}", strip_ext(&entry).display())?;
        }
        Ok(())
    }
}

pub trait VaultExt {
    fn select(&self, vault_id: &str) -> Result<Vault, Error>;
}

impl VaultExt for Vec<Vault> {
    fn select(&self, vault_id: &str) -> Result<Vault, Error> {
        let idx: Result<usize, _> = vault_id.parse();
        Ok(
            match idx {
                Ok(idx) => {
                    self.get(idx).ok_or_else(|| {
                        format_err!("Vault index {} is out of bounds.", idx)
                    })?
                }
                Err(_) => {
                    self.iter()
                        .find(|v| match v.name {
                            Some(ref name) if name == vault_id => true,
                            _ => false,
                        })
                        .ok_or_else(|| format_err!("Vault name '{}' is unknown.", vault_id))?
                }
            }.clone(),
        )
    }
}

fn normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut p = p.components().fold(PathBuf::new(), |mut p, c| {
        match c {
            Component::CurDir => {}
            _ => p.push(c.as_os_str()),
        }
        p
    });
    if p.components().count() == 0 {
        p = PathBuf::from(".");
    }
    p
}
#[cfg(test)]
mod tests_utils {
    use super::*;

    #[test]
    fn it_will_always_remove_current_dirs_including_the_first_one() {
        assert_eq!(
            format!("{}", normalize(Path::new("./././a")).display()),
            "a"
        )
    }
    #[test]
    fn it_does_not_alter_parent_dirs() {
        assert_eq!(
            format!("{}", normalize(Path::new("./../.././a")).display()),
            "../../a"
        )
    }
}

#[cfg(test)]
mod tests_vault {
    use super::*;

    #[test]
    fn it_print_the_name_in_the_url_if_there_is_none() {
        let mut v = Vault::default();
        v.name = Some("name".into());
        assert_eq!(v.url(), "syv://name@.")
    }

    #[test]
    fn it_does_not_print_the_name_in_the_url_if_there_is_none() {
        let v = Vault::default();
        assert_eq!(v.url(), "syv://.")
    }
}

#[cfg(test)]
mod tests_vault_ext {
    use super::*;

    #[test]
    fn it_selects_by_name() {
        let vault = Vault {
            name: Some("foo".into()),
            ..Default::default()
        };
        let v = vec![vault.clone()];
        assert_eq!(v.select("foo").unwrap(), vault)
    }

    #[test]
    fn it_selects_by_index() {
        let v = vec![Vault::default()];
        assert!(v.select("0").is_ok())
    }

    #[test]
    fn it_errors_if_name_is_unknown() {
        let v = Vec::<Vault>::new();
        assert_eq!(
            format!("{}", v.select("foo").unwrap_err()),
            "Vault name 'foo' is unknown."
        )
    }
    #[test]
    fn it_errors_if_index_is_out_of_bounds() {
        let v = Vec::<Vault>::new();
        assert_eq!(
            format!("{}", v.select("0").unwrap_err()),
            "Vault index 0 is out of bounds."
        )
    }
}
