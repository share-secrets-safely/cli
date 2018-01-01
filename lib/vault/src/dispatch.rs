extern crate sheesy_types;

use vault::{Vault, VaultExt};
use sheesy_types::VaultContext;
use failure::Error;
use sheesy_types::WriteMode;
use sheesy_types::Destination;
use std::io::Write;

fn vault_from(ctx: &VaultContext) -> Result<Vault, Error> {
    Vault::from_file(&ctx.vault_path)?.select(&ctx.vault_id)
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: VaultContext, output: &mut Write) -> Result<String, Error> {
    use sheesy_types::VaultCommand;
    match &ctx.command {
        &VaultCommand::RecipientsAdd { ref gpg_key_ids } => vault_from(&ctx)?
            .add_recipients(gpg_key_ids, output)
            .map(|_| String::new()),
        &VaultCommand::RecipientsInit { ref gpg_key_ids } => vault_from(&ctx)?
            .init_recipients(gpg_key_ids, output)
            .map(|_| String::new()),
        &VaultCommand::Init {
            ref gpg_key_ids,
            ref gpg_keys_dir,
            ref recipients_file,
            ref at,
        } => {
            Vault::init(
                &at,
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
        &VaultCommand::ResourceAdd { ref specs } => vault_from(&ctx)?.encrypt(
            &specs,
            WriteMode::RefuseOverwrite,
            Destination::ReolveAndAppendGpg,
        ),
        &VaultCommand::ResourceEdit {
            ref spec,
            ref editor,
            ref mode,
        } => vault_from(&ctx)?.edit(spec, editor, mode),
        &VaultCommand::List => vault_from(&ctx)?.list(output).map(|_| String::new()),
        &VaultCommand::ResourceShow { ref spec } => vault_from(&ctx)?
            .decrypt(spec, output)
            .map(|_| String::new()),
    }
}
