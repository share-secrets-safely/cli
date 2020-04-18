use crate::base::{Vault, VaultKind};
use crate::init::assure_empty_directory_exists;
use crate::spec::WriteMode;
use crate::util::{export_key_with_progress, extract_at_least_one_secret_key, fingerprint_of, new_context};
use failure::{err_msg, Error, ResultExt};
use std::io::Write;
use std::iter::once;
use std::path::Path;

impl Vault {
    pub fn all_in_order(&self) -> Vec<&Vault> {
        let mut all_vaults: Vec<_> = self.partitions.iter().chain(once(self)).collect();
        all_vaults.sort_by_key(|v| v.index);
        all_vaults
    }

    pub fn serialize(&self) -> Result<(), Error> {
        self.to_file(
            self.vault_path
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| err_msg("Expected vault to know its configuration file"))?,
            WriteMode::AllowOverwrite,
        )
        .map_err(Into::into)
    }

    pub fn partition_index<'a, I>(selector: &str, partitions: I, leader_index: Option<usize>) -> Result<usize, Error>
    where
        I: IntoIterator<Item = &'a Vault>,
    {
        let index: Result<usize, _> = selector.parse();
        Ok(match index {
            Ok(index) => {
                if let Some(leader_index) = leader_index {
                    if leader_index == index {
                        bail!("Refusing to remove the leading partition at index {}", index)
                    }
                };
                partitions
                    .into_iter()
                    .find(|v| v.index == index)
                    .map(|v| v.index)
                    .ok_or_else(|| format_err!("Could not find partition with index {}", index))?
            }
            Err(_) => {
                let selector_as_path = Path::new(selector);
                let mut matches = partitions.into_iter().filter_map(|v| {
                    if v.secrets.as_path() == selector_as_path {
                        Some(v.index)
                    } else {
                        v.name
                            .as_ref()
                            .and_then(|n| if n == selector { Some(v.index) } else { None })
                    }
                });
                match (matches.next(), matches.next()) {
                    (Some(index), None) => index,
                    (Some(_), Some(_)) => bail!("Multiple partitions matched the ambiguous selector '{}'", selector),
                    _ => bail!("No partition matched the given selector '{}'", selector),
                }
            }
        })
    }

    pub fn remove_partition(&mut self, selector: &str, output: &mut dyn Write) -> Result<(), Error> {
        let index = Vault::partition_index(selector, &self.partitions, Some(self.index))?;

        self.partitions.retain(|v| v.index != index);
        self.serialize()?;

        writeln!(output, "Removed partition matching selector '{}'", selector).ok();
        Ok(())
    }

    pub fn add_partition(
        &mut self,
        path: &Path,
        name: Option<&str>,
        gpg_key_ids: &[String],
        recipients_file: Option<&Path>,
        output: &mut dyn Write,
    ) -> Result<(), Error> {
        let secrets_dir = self.secrets.parent().ok_or_else(|| {
            format_err!(
                "Expected vault to have secrets directory ('{}') from which a parent directory can be obtained.",
                self.secrets.display()
            )
        })?;
        let partition_secrets_dir = secrets_dir.join(path);
        let recipients_file = match recipients_file {
            Some(p) => p.to_owned(),
            None => partition_secrets_dir.join(self.recipients.file_name().ok_or_else(|| {
                format_err!(
                    "Expected vault to have a recipients file ('{}') from which a filename can be obtained",
                    self.recipients.display()
                )
            })?),
        };
        let max_index = self
            .partitions
            .iter()
            .map(|v| v.index)
            .chain(once(self.index))
            .max()
            .expect("at least one item");
        let new_partition = Vault {
            name: name
                .map(ToOwned::to_owned)
                .or_else(|| path.file_name().map(|f| f.to_string_lossy().into_owned())),
            kind: VaultKind::Partition,
            index: max_index + 1,
            partitions: Vec::new(),
            resolved_at: self.resolved_at.clone(),
            vault_path: self.vault_path.clone(),
            secrets: partition_secrets_dir.clone(),
            gpg_keys: None,
            recipients: recipients_file,
            trust_model: None,
            auto_import: None,
        };

        let partition = new_partition.clone();
        self.partitions.push(new_partition);
        self.serialize()?;

        {
            let mut gpg_ctx = new_context()?;
            let keys = extract_at_least_one_secret_key(&mut gpg_ctx, gpg_key_ids)?;
            let mut fprs: Vec<_> = keys.iter().map(|k| fingerprint_of(k)).collect::<Result<_, _>>()?;
            assure_empty_directory_exists(&partition_secrets_dir).context("Cannot create secrets directory")?;
            partition.write_recipients_list(&mut fprs)?;

            if let Ok(gpg_keys_dir) = self.find_gpg_keys_dir() {
                let mut buf = Vec::new();
                for key in &keys {
                    export_key_with_progress(&mut gpg_ctx, &gpg_keys_dir, key, &mut buf, output)?;
                }
            }
        }

        match name {
            Some(name) => writeln!(
                output,
                "Added partition '{}' with resources at '{}'",
                name,
                partition_secrets_dir.display()
            ),
            None => writeln!(
                output,
                "Added unnamed partition with resources at '{}'",
                partition_secrets_dir.display()
            ),
        }
        .ok();

        Ok(())
    }
}
