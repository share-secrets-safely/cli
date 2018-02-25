use base::{Vault, VaultKind};
use failure::Error;
use std::io::Write;
use std::path::Path;
use sheesy_types::WriteMode;

impl Vault {
    pub fn add_partition(&mut self, path: &Path, name: Option<&str>, output: &mut Write) -> Result<(), Error> {
        let secrets_dir = self.secrets.parent().ok_or_else(|| {
            format_err!(
                "Expected vault to have secrets directory ('{}') from which a parent directory can be obtained.",
                self.secrets.display()
            )
        })?;
        let partition_secrets_dir = secrets_dir.join(path);
        let recipients_file = partition_secrets_dir.join(self.recipients.file_name().ok_or_else(|| {
            format_err!(
                "Expected vault to have a recipients file ('{}') from which a filename can be obtained",
                self.recipients.display()
            )
        })?);
        self.partitions.push(Vault {
            name: name.map(ToOwned::to_owned)
                .or_else(|| path.file_name().map(|f| f.to_string_lossy().into_owned())),
            kind: VaultKind::Partition,
            partitions: Vec::new(),
            resolved_at: self.resolved_at.clone(),
            vault_path: self.vault_path.clone(),
            secrets: partition_secrets_dir.clone(),
            gpg_keys: None,
            recipients: recipients_file,
        });

        self.to_file(
            self.vault_path
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| format_err!("Expected vault to know its configuration file"))?,
            WriteMode::AllowOverwrite,
        )?;

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
        }.ok();
        Ok(())
    }
}
