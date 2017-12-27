use vault::{Vault, VaultExt};
use s3_types::VaultContext;
use failure::Error;
use std::io::stdout;

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: VaultContext) -> Result<String, Error> {
    use s3_types::VaultCommand;
    match ctx.command {
        VaultCommand::Init {
            gpg_key_ids,
            gpg_keys_dir,
            recipients_file,
        } => {
            Vault::init(
                &gpg_key_ids,
                &gpg_keys_dir,
                &recipients_file,
                &ctx.vault_path,
                {
                    let r: Result<usize, _> = ctx.vault_id.parse();
                    match r {
                        Err(_) => Some(ctx.vault_id),
                        Ok(_) => None,
                    }
                },
            )?;
            Ok(format!(
                "vault initialized at '{}'",
                ctx.vault_path.display()
            ))
        }
        VaultCommand::ResourceAdd { specs } => Vault::from_file(&ctx.vault_path)?
            .select(&ctx.vault_id)?
            .add(&specs),
        VaultCommand::List => {
            let stdout = stdout();
            let mut lock = stdout.lock();
            Vault::from_file(&ctx.vault_path)?
                .select(&ctx.vault_id)?
                .list(&mut lock)?;
            Ok(String::new())
        }
    }
}
