use vault::{Vault, VaultExt};
use s3_types::VaultContext;
use failure::Error;
use std::io::stdout;

fn vault_from(ctx: &VaultContext) -> Result<Vault, Error> {
    Vault::from_file(&ctx.vault_path)?.select(&ctx.vault_id)
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: VaultContext) -> Result<String, Error> {
    use s3_types::VaultCommand;
    match &ctx.command {
        &VaultCommand::Init {
            ref gpg_key_ids,
            ref gpg_keys_dir,
            ref recipients_file,
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
        &VaultCommand::ResourceAdd { ref specs } => vault_from(&ctx)?.add(&specs),
        s @ &VaultCommand::List | s @ &VaultCommand::ResourceShow { .. } => {
            let vault = vault_from(&ctx)?;
            let stdout = stdout();
            let mut lock = stdout.lock();
            match s {
                &VaultCommand::List => vault.list(&mut lock),
                &VaultCommand::ResourceShow { ref spec } => vault.show(spec, &mut lock),
                _ => unreachable!(),
            }.map(|_| String::new())
        }
    }
}
