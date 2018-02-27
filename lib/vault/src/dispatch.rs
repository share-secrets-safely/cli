extern crate sheesy_types;

use base::{Vault, VaultExt};
use sheesy_types::VaultContext;
use failure::Error;
use sheesy_types::WriteMode;
use sheesy_types::Destination;
use std::io::Write;
use gpgme;
use error::first_cause_of_type;

fn vault_from(ctx: &VaultContext) -> Result<Vault, Error> {
    Vault::from_file(&ctx.vault_path)?.select(&ctx.vault_selector)
}

fn inner_do_it(ctx: VaultContext, output: &mut Write) -> Result<(), Error> {
    use sheesy_types::VaultCommand;
    match ctx.command {
        VaultCommand::PartitionsRemove { ref selector } => vault_from(&ctx)?.remove_partition(selector, output),
        VaultCommand::PartitionsAdd {
            ref path,
            ref name,
            ref gpg_key_ids,
        } => vault_from(&ctx)?.add_partition(path, name.as_ref().map(|s| s.as_str()), gpg_key_ids, output),
        VaultCommand::RecipientsRemove { ref gpg_key_ids } => vault_from(&ctx)?.remove_recipients(gpg_key_ids, output),
        VaultCommand::RecipientsAdd {
            ref gpg_key_ids,
            ref sign,
            ref signing_key_id,
        } => vault_from(&ctx)?.add_recipients(
            gpg_key_ids,
            *sign,
            signing_key_id.as_ref().map(String::as_str),
            output,
        ),
        VaultCommand::RecipientsList => vault_from(&ctx)?.list_recipients(output),
        VaultCommand::RecipientsInit { ref gpg_key_ids } => vault_from(&ctx)?.init_recipients(gpg_key_ids, output),
        VaultCommand::Init {
            ref name,
            ref gpg_key_ids,
            ref gpg_keys_dir,
            ref recipients_file,
            ref secrets,
        } => {
            Vault::init(
                secrets,
                gpg_key_ids,
                gpg_keys_dir,
                recipients_file,
                &ctx.vault_path,
                name.clone(),
                output,
            )?;
            Ok(())
        }
        VaultCommand::ResourceRemove { ref specs } => vault_from(&ctx)?.remove(specs, output),
        VaultCommand::ResourceAdd { ref specs } => vault_from(&ctx)?.encrypt(
            specs,
            WriteMode::RefuseOverwrite,
            Destination::ReolveAndAppendGpg,
            output,
        ),
        VaultCommand::ResourceEdit {
            ref spec,
            try_encrypt,
            ref editor,
            ref mode,
        } => vault_from(&ctx)?.edit(spec, editor, mode, try_encrypt, output),
        VaultCommand::List => vault_from(&ctx)?.list(output),
        VaultCommand::ResourceShow { ref spec } => vault_from(&ctx)?.decrypt(spec, output).map(|_| ()),
    }
}

/// A universal handler which delegates all functionality based on the provided Context
/// The latter is usually provided by the user interface.
pub fn do_it(ctx: VaultContext, output: &mut Write) -> Result<(), Error> {
    inner_do_it(ctx, output).map_err(|failure| {
        let gpg_error_code = match first_cause_of_type::<gpgme::Error>(&failure) {
            Some(gpg_err) => Some(gpg_err.code()),
            None => None, // failure.into(),
        };
        match gpg_error_code {
            Some(code) if code == gpgme::Error::NOT_SUPPORTED.code() => failure
                .context(
                    "The GNU Privacy Guard (GPG) does not supported the attempted operation.\n\
                     GPG v2 is known to work, and you can install it here:\n\
                     https://www.gnupg.org for more information.",
                )
                .into(),
            Some(code) if code == gpgme::Error::UNSUPPORTED_PROTOCOL.code() => failure
                .context(
                    "The GNU Privacy Guard (GPG) is not available on your system.\n\
                     Please install it and try again.\n\
                     See https://www.gnupg.org for more information.",
                )
                .into(),
            _ => failure,
        }
    })
}
