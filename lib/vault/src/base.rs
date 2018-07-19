use error::{IOMode, VaultError};
use failure::{err_msg, Error, ResultExt};
use glob::glob;
use gpgme;
use serde_yaml;
use spec::WriteMode;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::io::{stdin, BufRead, BufReader, Read, Write};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use util::{strip_ext, write_at, FingerprintUserId, ResetCWD};

pub const GPG_GLOB: &str = "**/*.gpg";
pub fn recipients_default() -> PathBuf {
    PathBuf::from(".gpg-id")
}

pub fn secrets_default() -> PathBuf {
    PathBuf::from(".")
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Hash)]
pub enum VaultKind {
    Leader,
    Partition,
}

impl Default for VaultKind {
    fn default() -> Self {
        VaultKind::Leader
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum TrustModel {
    GpgWebOfTrust,
    Always,
}

impl Default for TrustModel {
    fn default() -> Self {
        TrustModel::GpgWebOfTrust
    }
}

impl FromStr for TrustModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(match s {
            "web-of-trust" => TrustModel::GpgWebOfTrust,
            "always" => TrustModel::Always,
            _ => return Err(format!("Unknown trust model: '{}'", s)),
        })
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct Vault {
    pub name: Option<String>,
    #[serde(skip)]
    pub kind: VaultKind,
    #[serde(skip)]
    pub index: usize,
    #[serde(skip)]
    pub partitions: Vec<Vault>,
    #[serde(skip)]
    pub resolved_at: PathBuf,
    #[serde(skip)]
    pub vault_path: Option<PathBuf>,
    #[serde(default)]
    pub auto_import: Option<bool>,
    #[serde(default)]
    pub trust_model: Option<TrustModel>,
    #[serde(default = "secrets_default")]
    pub secrets: PathBuf,
    pub gpg_keys: Option<PathBuf>,
    #[serde(default = "recipients_default")]
    pub recipients: PathBuf,
}

impl Default for Vault {
    fn default() -> Self {
        Vault {
            kind: VaultKind::default(),
            index: 0,
            partitions: Default::default(),
            trust_model: Default::default(),
            auto_import: Some(true),
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
            if !path.exists() {
                if let Some(recipients_path) = path.parent().map(|p| p.join(recipients_default())) {
                    if recipients_path.is_file() {
                        let mut vault = Vault {
                            name: None,
                            kind: VaultKind::Leader,
                            index: 0,
                            partitions: Vec::new(),
                            resolved_at: path.to_owned(),
                            vault_path: None,
                            secrets: PathBuf::from("."),
                            gpg_keys: None,
                            recipients: recipients_default(),
                            auto_import: Some(false),
                            trust_model: Some(TrustModel::GpgWebOfTrust),
                        };
                        vault = vault.set_resolved_at(&recipients_path
                            .parent()
                            .expect("parent dir for recipient path which was joined before")
                            .join("sy-vault.yml"))?;
                        return Ok(vec![vault]);
                    }
                }
            }
            Box::new(File::open(path).map_err(|cause| VaultError::from_io_err(cause, path, &IOMode::Read))?)
        };
        let vaults: Vec<_> = split_documents(reader)?
            .iter()
            .enumerate()
            .map(|(index, s)| {
                serde_yaml::from_str(s)
                    .map_err(|cause| VaultError::Deserialization {
                        cause,
                        path: path.to_owned(),
                    })
                    .map_err(Into::into)
                    .and_then(|v: Vault| match v.set_resolved_at(path) {
                        Ok(mut v) => {
                            v.index = index;
                            Ok(v)
                        }
                        err => err,
                    })
            })
            .collect::<Result<_, _>>()?;
        if !vaults.is_empty() {
            vaults[0].validate()?;
        }
        Ok(vaults)
    }

    pub fn set_resolved_at(mut self, vault_file: &Path) -> Result<Self, Error> {
        self.resolved_at = normalize(vault_file
            .parent()
            .ok_or_else(|| format_err!("The vault file path '{}' is invalid.", vault_file.display()))?);
        self.vault_path = Some(vault_file.to_owned());
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.partitions.is_empty() {
            return Ok(());
        }
        {
            let all_secrets_paths: Vec<_> = self.partitions
                .iter()
                .map(|v| v.secrets_path())
                .chain(once(self.secrets_path()))
                .map(|mut p| {
                    if p.is_relative() {
                        p = Path::new(".").join(p);
                    }
                    p
                })
                .collect();
            for (sp, dp) in iproduct!(
                all_secrets_paths.iter().enumerate(),
                all_secrets_paths.iter().enumerate()
            ).filter_map(|((si, s), (di, d))| if si == di { None } else { Some((s, d)) })
            {
                if sp.starts_with(&dp) {
                    bail!(
                        "Partition at '{}' is contained in another partitions resources directory at '{}'",
                        sp.display(),
                        dp.display()
                    );
                }
            }
        }
        {
            let mut seen: HashSet<_> = Default::default();
            for path in self.partitions
                .iter()
                .map(|v| v.recipients_path())
                .chain(once(self.recipients_path()))
            {
                if seen.contains(&path) {
                    bail!(
                        "Recipients path '{}' is already used, but must be unique across all partitions",
                        path.display()
                    );
                }
                seen.insert(path);
            }
        }

        Ok(())
    }

    pub fn to_file(&self, path: &Path, mode: WriteMode) -> Result<(), VaultError> {
        if let WriteMode::RefuseOverwrite = mode {
            if path.exists() {
                return Err(VaultError::ConfigurationFileExists(path.to_owned()));
            }
        }
        self.validate().map_err(VaultError::Validation)?;

        match self.kind {
            VaultKind::Partition => return Err(VaultError::PartitionUnsupported),
            VaultKind::Leader => {
                let mut file = write_at(path).map_err(|cause| VaultError::from_io_err(cause, path, &IOMode::Write))?;
                let all_vaults = self.all_in_order();
                for vault in &all_vaults {
                    serde_yaml::to_writer(&mut file, vault)
                        .map_err(|cause| VaultError::Serialization {
                            cause,
                            path: path.to_owned(),
                        })
                        .and_then(|_| {
                            writeln!(file).map_err(|cause| VaultError::from_io_err(cause, path, &IOMode::Write))
                        })?;
                }
            }
        }
        Ok(())
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

    pub fn print_resources(&self, w: &mut Write) -> Result<(), Error> {
        let has_multiple_partitions = !self.partitions.is_empty();
        for partition in once(self).chain(self.partitions.iter()) {
            writeln!(w, "{}", partition.url())?;
            let dir = partition.secrets_path();
            if !dir.is_dir() {
                continue;
            }
            let _change_cwd = ResetCWD::new(&dir)?;
            for entry in glob(GPG_GLOB).expect("valid pattern").filter_map(Result::ok) {
                if has_multiple_partitions {
                    writeln!(w, "{}", dir.join(strip_ext(&entry)).display())?;
                } else {
                    writeln!(w, "{}", strip_ext(&entry).display())?;
                }
            }
        }
        Ok(())
    }

    pub fn write_recipients_list(&self, recipients: &mut Vec<String>) -> Result<PathBuf, Error> {
        recipients.sort();
        recipients.dedup();

        let recipients_path = self.recipients_path();
        if let Some(recipients_parent_dir) = recipients_path.parent() {
            if !recipients_parent_dir.is_dir() {
                create_dir_all(recipients_parent_dir).context(format!(
                    "Failed to create directory leading to recipients file at '{}'",
                    recipients_path.display()
                ))?;
            }
        }
        let mut writer = write_at(&recipients_path).context(format!(
            "Failed to open recipients at '{}' file for writing",
            recipients_path.display()
        ))?;
        for recipient in recipients {
            writeln!(&mut writer, "{}", recipient).context(format!(
                "Failed to write recipient '{}' to file at '{}'",
                recipient,
                recipients_path.display()
            ))?
        }
        Ok(recipients_path)
    }

    pub fn recipients_path(&self) -> PathBuf {
        self.absolute_path(&self.recipients)
    }

    pub fn recipients_list(&self) -> Result<Vec<String>, Error> {
        let recipients_file_path = self.recipients_path();
        let rfile = File::open(&recipients_file_path).map(BufReader::new).context(format!(
            "Could not open recipients file at '{}' for reading",
            recipients_file_path.display()
        ))?;
        Ok(rfile.lines().collect::<Result<_, _>>().context(format!(
            "Could not read all recipients from file at '{}'",
            recipients_file_path.display()
        ))?)
    }

    pub fn keys_by_ids(
        &self,
        ctx: &mut gpgme::Context,
        ids: &[String],
        type_of_ids_for_errors: &str,
        gpg_keys_dir: Option<&Path>,
        output: &mut io::Write,
    ) -> Result<Vec<gpgme::Key>, Error> {
        ctx.find_keys(ids)
            .context(format!("Could not iterate keys for given {}s", type_of_ids_for_errors))?;
        let (keys, missing): (Vec<gpgme::Key>, Vec<String>) = ids.iter().map(|id| (ctx.get_key(id), id)).fold(
            (Vec::new(), Vec::new()),
            |(mut keys, mut missing), (r, id)| {
                match r {
                    Ok(k) => keys.push(k),
                    Err(_) => missing.push(id.to_owned()),
                };
                (keys, missing)
            },
        );
        if keys.len() == ids.len() {
            assert_eq!(missing.len(), 0);
            return Ok(keys);
        }
        let diff: isize = ids.len() as isize - keys.len() as isize;
        let mut msg = vec![if diff > 0 {
            if let Some(dir) = gpg_keys_dir {
                self.import_keys(ctx, dir, &missing, output)
                    .context("Could not auto-import all required keys")?;
                return self.keys_by_ids(ctx, ids, type_of_ids_for_errors, None, output);
            }

            let mut msg = format!(
                "Didn't find the key for {} {}(s) in the gpg database.{}",
                diff,
                type_of_ids_for_errors,
                match self.gpg_keys.as_ref() {
                    Some(dir) => format!(
                        " This might mean it wasn't imported yet from the '{}' directory.",
                        self.absolute_path(dir).display()
                    ),
                    None => String::new(),
                }
            );
            msg.push_str(&format!(
                "\nThe following {}(s) could not be found in the gpg key database:",
                type_of_ids_for_errors
            ));
            for fpr in missing {
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
                self.recipients_path().display()
            )
        }];
        if !keys.is_empty() {
            msg.push(format!("All {}s found in gpg database:", type_of_ids_for_errors));
            msg.extend(keys.iter().map(|k| format!("{}", FingerprintUserId(k))));
        }
        Err(err_msg(msg.join("\n")))
    }

    pub fn recipient_keys(
        &self,
        ctx: &mut gpgme::Context,
        gpg_keys_dir: Option<&Path>,
        output: &mut io::Write,
    ) -> Result<Vec<gpgme::Key>, Error> {
        let recipients_fprs = self.recipients_list()?;
        if recipients_fprs.is_empty() {
            return Err(format_err!(
                "No recipients found in recipients file at '{}'.",
                self.recipients.display()
            ));
        }
        self.keys_by_ids(ctx, &recipients_fprs, "recipient", gpg_keys_dir, output)
    }

    fn vault_path_for_display(&self) -> String {
        self.vault_path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("<unknown>"))
    }

    /// TODO: change this to be similar to `find_trust_model()`
    /// as it's OK to let partitions override the master vault settings
    pub fn find_gpg_keys_dir(&self) -> Result<PathBuf, Error> {
        once(self.gpg_keys_dir())
            .chain(self.partitions.iter().map(|p| p.gpg_keys_dir()))
            .filter_map(Result::ok)
            .next()
            .ok_or_else(|| {
                format_err!(
                    "The vault at '{}' does not have a gpg_keys directory configured.",
                    self.vault_path_for_display()
                )
            })
    }

    pub fn gpg_keys_dir_for_auto_import(&self, partition: &Vault) -> Option<PathBuf> {
        let auto_import = partition.auto_import.or(self.auto_import).unwrap_or(false);
        if auto_import {
            self.find_gpg_keys_dir().ok()
        } else {
            None
        }
    }

    pub fn gpg_keys_dir(&self) -> Result<PathBuf, Error> {
        self.gpg_keys.as_ref().map(|p| self.absolute_path(p)).ok_or_else(|| {
            format_err!(
                "The vault at '{}' does not have a gpg_keys directory configured.",
                self.vault_path_for_display()
            )
        })
    }
}

pub trait VaultExt {
    fn select(self, vault_id: &str) -> Result<Vault, Error>;
}

impl VaultExt for Vec<Vault> {
    fn select(mut self, selector: &str) -> Result<Vault, Error> {
        let leader_index = Vault::partition_index(selector, self.iter(), None)?;
        for (_, vault) in self.iter_mut().enumerate().filter(|&(vid, _)| vid != leader_index) {
            vault.kind = VaultKind::Partition;
        }

        let mut vault = self[leader_index].clone();
        vault.kind = VaultKind::Leader;

        self.retain(|v| match v.kind {
            VaultKind::Partition => true,
            VaultKind::Leader => false,
        });

        vault.partitions = self;
        Ok(vault)
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

    let docs = YamlLoader::load_from_str(&buf).context("YAML deserialization failed")?;
    Ok(docs.iter()
        .map(|d| {
            let mut out_str = String::new();
            {
                let mut emitter = YamlEmitter::new(&mut out_str);
                emitter.dump(d).expect("dumping a valid yaml into a string to work");
            }
            out_str
        })
        .collect())
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
    fn it_selects_by_secrets_dir() {
        let vault = Vault {
            secrets: PathBuf::from("../dir"),
            ..Default::default()
        };
        let v = vec![vault.clone()];
        assert_eq!(v.select("../dir").unwrap(), vault)
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
            "No partition matched the given selector 'foo'"
        )
    }
    #[test]
    fn it_errors_if_index_is_out_of_bounds() {
        let v = Vec::<Vault>::new();
        assert_eq!(
            format!("{}", v.select("0").unwrap_err()),
            "Could not find partition with index 0"
        )
    }
}

#[cfg(test)]
mod tests_utils {
    use super::*;

    #[test]
    fn it_will_always_remove_current_dirs_including_the_first_one() {
        assert_eq!(format!("{}", normalize(Path::new("./././a")).display()), "a")
    }
    #[test]
    fn it_does_not_alter_parent_dirs() {
        assert_eq!(format!("{}", normalize(Path::new("./../.././a")).display()), "../../a")
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
