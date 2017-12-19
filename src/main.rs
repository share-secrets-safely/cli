#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate s3_extract as extract;
extern crate s3_types as types;
extern crate s3_vault as vault;

use types::{ExtractionContext, VaultCommand, VaultContext};
use clap::{App, Arg, ArgMatches, Shell};
use failure::{err_msg, Error, ResultExt};
use std::io::{stderr, stdout, Write};
use std::env;
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::convert::Into;

const CLI_NAME: &'static str = "s3";

fn required_arg<'a, T>(args: &'a ArgMatches, name: &'static str) -> Result<T, Error>
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
        vault_path: required_arg(args, "config-file")?,
        command: VaultCommand::List,
    })
}
fn vault_init_from(ctx: VaultContext, args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        command: VaultCommand::Init {
            gpg_keyfile_path: args.value_of("gpg-keyfile").map(Into::into),
            gpg_key_id: args.value_of("gpg-key-id").map(Into::into),
        },
        ..ctx
    })
}

fn extraction_context_from(args: &ArgMatches) -> Result<ExtractionContext, Error> {
    Ok(ExtractionContext {
        file_path: required_arg(args, "file")?,
    })
}

fn generate_completions(mut app: App, args: &ArgMatches) -> Result<(), Error> {
    let shell = args.value_of("shell")
        .ok_or(err_msg("expected 'shell' argument"))
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
    Ok(())
}

fn usage_and_exit(args: &ArgMatches) -> ! {
    println!("{}", args.usage());
    process::exit(1)
}

fn main() {
    let shell = env::var("SHELL");
    let app: App = app_from_crate!()
        .name(CLI_NAME)
        .subcommand(
            App::new("completions")
                .about("generate completions for supported shell")
                .arg({
                    let arg = Arg::with_name("shell").required(shell.is_err()).help(
                        "The name of the shell, or the path to the shell as exposed by the \
                         $SHELL variable.",
                    );
                    if let Ok(shell) = shell.as_ref() {
                        arg.default_value(&shell)
                    } else {
                        arg
                    }
                }),
        )
        .subcommand(
            App::new("vault")
                .about("a variety of vault interactions")
                .subcommand(
                    App::new("init")
                        .about("initialize the vault")
                        .help(
                            "If neither --gpg-keyfile nor --gpg-key-id are set, we will use the \
                             only key that you have a secret key for.\
                             If you have multiple keys, the --gpg-key-id must be specified \
                             to make the input \
                             unambiguous.",
                        )
                        .arg(
                            Arg::with_name("gpg-key-id")
                                .short("i")
                                .required(false)
                                .help(
                                    "The key-id of the public key identifying your own user \
                                     identifying the your own user.",
                                ),
                        )
                        .arg(
                            Arg::with_name("gpg-keyfile")
                                .short("k")
                                .required(false)
                                .help(
                                    "Path to a keyfile exported with 'gpg --export --armor ...' \
                                     identifying the your own user.",
                                ),
                        ),
                )
                .arg(
                    Arg::with_name("config-file")
                        .short("c")
                        .required(true)
                        .help("Path to the vault configuration file.")
                        .default_value("./s3-vault.yml"),
                ),
        )
        .subcommand(
            App::new("extract")
                .about("utilities to extract information from structured data files")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .required(true)
                        .help("Path to the data file to read"),
                ),
        );

    let appc = app.clone();
    let matches: ArgMatches = app.get_matches();

    let res = match matches.subcommand() {
        ("completions", Some(args)) => generate_completions(appc, &args),
        ("vault", Some(args)) => {
            let mut context = ok_or_exit(vault_context_from(args));
            context = match args.subcommand() {
                ("init", Some(args)) => ok_or_exit(vault_init_from(context, args)),
                _ => usage_and_exit(&args),
            };
            vault::do_it(context)
        }
        ("extract", Some(args)) => {
            let context = ok_or_exit(extraction_context_from(args));
            extract::do_it(context)
        }
        _ => usage_and_exit(&matches),
    };
    ok_or_exit(res);
}
