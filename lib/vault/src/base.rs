use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, BufRead, stdin, Read, Write};
use serde_yaml;
use util::{FingerprintUserId, strip_ext, ResetCWD, write_at};
use error::{IOMode, VaultError};
use failure::{Error, ResultExt, err_msg};
use glob::glob;
use gpgme;

pub const GPG_GLOB: &str = "**/*.gpg";
pub fn recipients_default() -> PathBuf {
    PathBuf::from(".gpg-id")
}

pub fn secrets_default() -> PathBuf {
    PathBuf::from(".")
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

    pub fn write_recipients_list(&self, recipients: &mut Vec<String>) -> Result<PathBuf, Error> {
        recipients.sort();
        recipients.dedup();

        let recipients_file = self.absolute_path(&self.recipients);
        let mut writer = write_at(&recipients_file).context(format!(
            "Failed to open recipients at '{}' file for writing",
            recipients_file.display()
        ))?;
        for recipient in recipients {
            writeln!(&mut writer, "{}", recipient).context(format!(
                "Failed to write recipient '{}' to file at '{}'",
                recipient,
                recipients_file
                    .display()
            ))?
        }
        Ok(recipients_file)
    }

    pub fn recipients_list(&self) -> Result<Vec<String>, Error> {
        let recipients_file_path = self.absolute_path(&self.recipients);
        let rfile = File::open(&recipients_file_path)
            .map(BufReader::new)
            .context(format!(
                "Could not open recipients file at '{}' for reading",
                recipients_file_path.display()
            ))?;
        Ok(rfile.lines().collect::<Result<_, _>>().context(format!(
            "Could not read all recipients from file at '{}'",
            recipients_file_path
                .display()
        ))?)
    }

    pub fn keys_by_ids(
        &self,
        ctx: &mut gpgme::Context,
        ids: &[String],
        type_of_ids_for_errors: &str,
    ) -> Result<Vec<gpgme::Key>, Error> {
        let keys: Vec<gpgme::Key> = ctx.find_keys(ids)
            .context(format!(
                "Could not iterate keys for given {}s",
                type_of_ids_for_errors
            ))?
            .filter_map(Result::ok)
            .collect();
        if keys.len() == ids.len() {
            return Ok(keys);
        }
        let diff = ids.len() - keys.len();
        let mut msg = vec![
            if diff > 0 {
                // TODO: this check doesn't work anymore if ids is not full fingerprints
                // which now can happen.
                let existing_fprs: Vec<_> = keys.iter().filter_map(|k| k.fingerprint().ok()).collect();
                let missing_fprs = ids.iter().fold(Vec::new(), |mut m, f| {
                    if existing_fprs.iter().all(|of| of != f) {
                        m.push(f);
                    }
                    m
                });
                let mut msg = format!(
                    "Didn't find the key for {} {}(s) in the gpg database.{}",
                    diff,
                    type_of_ids_for_errors,
                    match self.gpg_keys.as_ref() {
                        Some(dir) => {
                            format!(
                                " This might mean it wasn't imported yet from the '{}' directory.",
                                self.absolute_path(dir).display()
                            )
                        }
                        None => String::new(),
                    }
                );
                msg.push_str(&format!(
                    "\nThe following {}(s) could not be found in the gpg key database:",
                    type_of_ids_for_errors
                ));
                for fpr in missing_fprs {
                    msg.push_str("\n");
                    let key_path_info = match self.gpg_keys.as_ref() {
                        Some(dir) => {
                            let key_path = self.absolute_path(dir).join(&fpr);
                            format!(
                                "{}'{}'",
                                if key_path.is_file() {
                                    "Import key-file using 'gpg --import "
                                } else {
                                    "Key-file does not exist at "
                                },
                                key_path.display()
                            )
                        }
                        None => "No GPG keys directory".into(),
                    };
                    msg.push_str(&format!("{} ({})", &fpr, key_path_info));
                }
                msg
            } else {
                format!(
                    "Found {} additional keys to encrypt for, which may indicate an unusual \
                        {}s specification in the recipients file at '{}'",
                    diff,
                    type_of_ids_for_errors,
                    self.absolute_path(&self.recipients).display()
                )
            },
        ];
        if !keys.is_empty() {
            msg.push(format!(
                "All {}s found in gpg database:",
                type_of_ids_for_errors
            ));
            msg.extend(keys.iter().map(|k| format!("{}", FingerprintUserId(k))));
        }
        return Err(err_msg(msg.join("\n")));
    }

    pub fn recipient_keys(&self, ctx: &mut gpgme::Context) -> Result<Vec<gpgme::Key>, Error> {
        let recipients_fprs = self.recipients_list()?;
        if recipients_fprs.is_empty() {
            return Err(format_err!(
                "No recipients found in recipients file at '{}'.",
                self.recipients.display()
            ));
        }
        self.keys_by_ids(ctx, &recipients_fprs, "recipient")
    }

    pub fn gpg_keys_dir(&self) -> Result<PathBuf, Error> {
        let unknown_path = PathBuf::from("<unknown>");
        self.gpg_keys
            .as_ref()
            .map(|p| self.absolute_path(p))
            .ok_or_else(|| {
                format_err!(
                    "The vault at '{}' does not have a gpg_keys directory configured.",
                    self.vault_path
                        .as_ref()
                        .unwrap_or_else(|| &unknown_path)
                        .display()
                )
            })
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
