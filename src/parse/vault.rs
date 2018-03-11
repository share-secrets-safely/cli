use conv::TryInto;
use failure::Error;
use clap::ArgMatches;

use std::path::{Path, PathBuf};
use std::convert::Into;

use vault::{CreateMode, SigningMode};
use dispatch::vault::{Command, Context};

use super::util::{optional_args, required_arg, required_os_arg};

pub fn context_from(args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        vault_path: required_os_arg(args, "config-file")?,
        vault_selector: required_arg(args, "vault-selector")?,
        command: Command::List,
    })
}

pub fn recipients_list(ctx: Context, _args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::RecipientsList,
        ..ctx
    })
}

pub fn recipients_init(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::RecipientsInit {
            gpg_key_ids: optional_args(args, "gpg-key-id"),
        },
        ..ctx
    })
}

pub fn recipients_remove(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::RecipientsRemove {
            partitions: optional_args(args, "partition"),
            gpg_key_ids: args.values_of("gpg-key-id")
                .expect("Clap to assure this is a required arg")
                .map(Into::into)
                .collect(),
        },
        ..ctx
    })
}

pub fn partitions_add(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    let recipients_file: Option<PathBuf> = args.value_of_os("recipients-file-path").map(Into::into);
    Ok(Context {
        command: Command::PartitionsAdd {
            recipients_file,
            gpg_key_ids: optional_args(args, "gpg-key-id"),
            path: required_os_arg(args, "partition-path")?,
            name: args.value_of("name").map(ToOwned::to_owned),
        },
        ..ctx
    })
}

pub fn partitions_remove(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::PartitionsRemove {
            selector: required_arg(args, "partition-selector")?,
        },
        ..ctx
    })
}

pub fn recipients_add(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::RecipientsAdd {
            sign: if args.is_present("verified") {
                SigningMode::None
            } else {
                SigningMode::Public
            },
            partitions: optional_args(args, "partition"),
            signing_key_id: args.value_of("signing-key").map(ToOwned::to_owned),
            gpg_key_ids: args.values_of("gpg-key-id")
                .expect("Clap to assure this is a required arg")
                .map(Into::into)
                .collect(),
        },
        ..ctx
    })
}

pub fn resource_show(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::ResourceShow {
            spec: required_os_arg(args, "path")?,
        },
        ..ctx
    })
}

pub fn resource_list(ctx: Context, _args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::List,
        ..ctx
    })
}

pub fn resource_edit(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::ResourceEdit {
            spec: required_os_arg(args, "path")?,
            editor: required_os_arg(args, "editor")?,
            try_encrypt: !args.is_present("no-try-encrypt"),
            mode: if args.is_present("no-create") {
                CreateMode::NoCreate
            } else {
                CreateMode::Create
            },
        },
        ..ctx
    })
}

pub fn vault_resource_remove(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::ResourceRemove {
            specs: match args.values_of("path") {
                Some(v) => v.map(PathBuf::from).collect(),
                None => Vec::new(),
            },
        },
        ..ctx
    })
}

pub fn resource_add(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        command: Command::ResourceAdd {
            specs: match args.values_of("spec") {
                Some(v) => v.map(|s| s.try_into()).collect::<Result<_, _>>()?,
                None => Vec::new(),
            },
        },
        ..ctx
    })
}

pub fn init_from(ctx: Context, args: &ArgMatches) -> Result<Context, Error> {
    let mut recipients_file: PathBuf = required_os_arg(args, "recipients-file-path")?;
    let secrets: PathBuf = required_os_arg(args, "secrets-dir")?;
    if args.is_present("first-partition") && secrets == Path::new(".") {
        bail!("If --first-partition is present, --secrets-dir must not be set to '.' or left unset");
    }
    if recipients_file.components().count() == 1 {
        recipients_file = secrets.join(recipients_file);
    }
    Ok(Context {
        command: Command::Init {
            name: args.value_of("name").map(ToOwned::to_owned),
            recipients_file,
            secrets,
            gpg_keys_dir: required_os_arg(args, "gpg-keys-dir")?,
            gpg_key_ids: optional_args(args, "gpg-key-id"),
        },
        ..ctx
    })
}
