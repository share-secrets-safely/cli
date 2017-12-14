#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate s3_vault as vault;
extern crate s3_types as types;

use types::VaultContext;
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
        None => Err(format_err!("expected clap argument '{}' to be set", name)),
    }
}

fn vault_context_from(args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        vault_path: required_arg(args, "config-file")?,
    })
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

fn main() {
    let app: App = app_from_crate!().subcommand(
        App::new("vault")
            .about("a variety of vault interactions")
            .arg(
                Arg::with_name("config-file")
                    .required(true)
                    .help("Path to the vault configuration file.")
                    .default_value("./s3-vault.yml"),
            ),
    );
    let matches: ArgMatches = app.get_matches();
    match matches.subcommand() {
        ("vault", Some(args)) => {
            let context = ok_or_exit(vault_context_from(args).context("context creation failed"));
            println!("Parsed opts");
            println!("{:?}", context);
        }
        _ => {
            println!("{}", matches.usage());
        }
    }
}
