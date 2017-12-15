#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate s3_extract as extract;
extern crate s3_types as types;
extern crate s3_vault as vault;

use types::{ExtractionContext, VaultContext};
use clap::{App, Arg, ArgMatches};
use failure::{Error, ResultExt};
use std::io::{stderr, Write};
use std::process;
use std::str::FromStr;
use std::convert::Into;

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
            for cause in e.causes() {
                writeln!(stderr(), "{}", cause).ok();
            }
            process::exit(1);
        }
    }
}

fn vault_context_from(args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        vault_path: required_arg(args, "config-file")?,
    })
}

fn extraction_context_from(args: &ArgMatches) -> Result<ExtractionContext, Error> {
    Ok(ExtractionContext {
        file_path: required_arg(args, "file")?,
    })
}

fn main() {
    let app: App = app_from_crate!()
        .subcommand(
            App::new("vault")
                .about("a variety of vault interactions")
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

    let matches: ArgMatches = app.get_matches();
    let res = match matches.subcommand() {
        ("vault", Some(args)) => {
            let context =
                ok_or_exit(vault_context_from(args).context("vault context creation failed"));
            vault::do_it(&context)
        }
        ("extract", Some(args)) => {
            let context = ok_or_exit(
                extraction_context_from(args).context("extraction context creation failed"),
            );
            extract::do_it(&context)
        }
        _ => {
            println!("{}", matches.usage());
            process::exit(2)
        }
    };
    ok_or_exit(res);
}
