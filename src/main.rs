#[macro_use]
extern crate clap;
extern crate conv;
#[macro_use]
extern crate failure;
extern crate s3_extract as extract;
extern crate s3_types as types;
extern crate s3_vault as vault;

use conv::TryInto;
use types::{ExtractionContext, VaultCommand, VaultContext};
use clap::{App, Arg, ArgMatches, Shell};
use failure::{err_msg, Error, ResultExt};
use std::io::{stderr, stdout, Write};
use std::env;
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::convert::Into;
use std::ffi::OsStr;

const CLI_NAME: &str = "s3";

fn required_os_arg<'a, T>(args: &'a ArgMatches, name: &'static str) -> Result<T, Error>
where
    T: From<&'a OsStr>,
{
    match args.value_of_os(name).map(Into::into) {
        Some(t) => Ok(t),
        None => Err(format_err!(
            "BUG: expected clap argument '{}' to be set",
            name
        )),
    }
}

fn _required_arg<T>(args: &ArgMatches, name: &'static str) -> Result<T, Error>
where
    T: FromStr,
    <T as FromStr>::Err: 'static + ::std::error::Error + Send + Sync,
{
    match args.value_of(name).map(FromStr::from_str) {
        Some(Ok(t)) => Ok(t),
        Some(Err(e)) => Err(e.into()),
        None => Err(format_err!(
            "BUG: expected clap argument '{}' to be set",
            name
        )),
    }
}

fn ok_or_exit<T, E>(r: Result<T, E>) -> T
where
    E: Into<Error>,
{
    match r {
        Ok(r) => r,
        Err(e) => {
            let e = e.into();
            let causes = e.causes().collect::<Vec<_>>();
            let num_causes = causes.len();
            for (index, cause) in causes.iter().enumerate() {
                if index == 0 {
                    writeln!(stderr(), "error: {}", cause).ok();
                    if num_causes > 1 {
                        writeln!(stderr(), "Caused by: ").ok();
                    }
                } else {
                    writeln!(stderr(), " {}: {}", num_causes - index, cause).ok();
                }
            }
            process::exit(1);
        }
    }
}

fn vault_context_from(args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        vault_path: required_os_arg(args, "config-file")?,
        command: VaultCommand::List,
    })
}

fn vault_resource_add_from(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::ResourceAdd {
            specs: match args.values_of("spec") {
                Some(v) => v.map(|s| s.try_into()).collect::<Result<_, _>>()?,
                None => Vec::new(),
            },
        },
        ..ctx
    })
}

fn vault_init_from(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::Init {
            recipients_file: required_os_arg(args, "recipients-file-path")?,
            gpg_keys_dir: required_os_arg(args, "gpg-keys-dir")?,
            gpg_key_ids: match args.values_of("gpg-key-id") {
                Some(v) => v.map(Into::into).collect(),
                None => Vec::new(),
            },
        },
        ..ctx
    })
}

fn extraction_context_from(args: &ArgMatches) -> Result<ExtractionContext, Error> {
    Ok(ExtractionContext {
        file_path: required_os_arg(args, "file")?,
    })
}

fn generate_completions(mut app: App, args: &ArgMatches) -> Result<String, Error> {
    let shell = args.value_of("shell")
        .ok_or_else(|| err_msg("expected 'shell' argument"))
        .map(|s| {
            Path::new(s)
                .file_name()
                .map(|f| f.to_str().expect("os-string to str conversion failed"))
                .unwrap_or(s)
        })
        .and_then(|s| {
            Shell::from_str(s)
                .map_err(err_msg)
                .context(format!("The shell '{}' is unsupported", s))
                .map_err(Into::into)
        })?;
    app.gen_completions_to(CLI_NAME, shell, &mut stdout());
    Ok(String::new())
}

fn usage_and_exit(args: &ArgMatches) -> ! {
    println!("{}", args.usage());
    process::exit(1)
}

fn main() {
    let shell = env::var("SHELL");
    let init = App::new("init")
        .about(
            "Initialize the vault in the current directory. \
             \
             If --gpg-key-id is unset, we will use the \
             only key that you have a secret key for, assuming it is yours.\
             If you have multiple keys, the --gpg-key-id must be specified \
             to make the input unambiguous.",
        )
        .arg(
            Arg::with_name("recipients-file-path")
                .long("recipients-file")
                .default_value(".gpg-id")
                .short("r")
                .required(false)
                .takes_value(true)
                .value_name("path")
                .help(
                    "The directory to hold the public keys identified by \
                     --gpg-key-id, with signatures.",
                ),
        )
        .arg(
            Arg::with_name("gpg-keys-dir")
                .long("gpg-keys-dir")
                .default_value(".gpg-keys")
                .short("k")
                .required(false)
                .takes_value(true)
                .value_name("directory")
                .help(
                    "The directory to hold the public keys identified by \
                     --gpg-key-id, with signatures.",
                ),
        )
        .arg(
            Arg::with_name("gpg-key-id")
                .long("gpg-key-id")
                .multiple(true)
                .short("i")
                .required(false)
                .takes_value(true)
                .value_name("userid")
                .help("The key-id of the public key identifying your own user(s)."),
        );

    let add_resource = App::new("add")
        .alias("insert")
        .about("Add a new resource to the vault.")
        .arg(
            Arg::with_name("spec")
                .required(true)
                .multiple(false)
                .takes_value(true)
                .value_name("spec")
                .help("A specification identifying a mapping from a source file to be stored \
                in a location of the vault. It takes the form '<src>:<dst>', where \
                '<src>' is equivalent to '<src>:<src>'.\
                <dst> should be vault-relative paths, whereas <src> must point to a readable file \
                and can be empty to read from standard input, such as in ':<dst>'."),
        );
    let resource = App::new("resource")
        .alias("contents")
        .about("Handle resources stored in your vault")
        .subcommand(add_resource);
    let vault = App::new("vault")
        .about("a variety of vault interactions")
        .subcommand(init)
        .subcommand(resource)
        .arg(
            Arg::with_name("config-file")
                .short("c")
                .required(true)
                .value_name("path")
                .help("Path to the vault configuration YAML file.")
                .default_value("./s3-vault.yml"),
        );
    let extract = App::new("extract")
        .about("utilities to extract information from structured data files")
        .arg(
            Arg::with_name("file")
                .short("f")
                .required(true)
                .help("Path to the data file to read"),
        );
    let completions = App::new("completions")
        .about("generate completions for supported shell")
        .arg({
            let arg = Arg::with_name("shell").required(shell.is_err()).help(
                "The name of the shell, or the path to the shell as exposed by the \
                 $SHELL variable.",
            );
            if let Ok(shell) = shell.as_ref() {
                arg.default_value(shell)
            } else {
                arg
            }
        });
    let app: App = app_from_crate!()
        .name(CLI_NAME)
        .subcommand(completions)
        .subcommand(vault)
        .subcommand(extract);

    let appc = app.clone();
    let matches: ArgMatches = app.get_matches();

    let res = match matches.subcommand() {
        ("completions", Some(args)) => generate_completions(appc, args),
        ("vault", Some(args)) => {
            let mut context = ok_or_exit(vault_context_from(args));
            context = match args.subcommand() {
                ("init", Some(args)) => ok_or_exit(vault_init_from(context, args)),
                ("resource", Some(args)) => match args.subcommand() {
                    ("add", Some(args)) => ok_or_exit(vault_resource_add_from(context, args)),
                    _ => usage_and_exit(&matches),
                },
                _ => context,
            };
            vault::do_it(context)
        }
        ("extract", Some(args)) => {
            let context = ok_or_exit(extraction_context_from(args));
            extract::do_it(context)
        }
        _ => usage_and_exit(&matches),
    };

    let msg = ok_or_exit(res);
    if !msg.is_empty() {
        ok_or_exit(writeln!(stdout(), "{}", msg));
    }
}
