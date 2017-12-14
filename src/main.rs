#[macro_use]
extern crate clap;
extern crate failure;

use clap::{App, Arg, ArgMatches};
use failure::{err_msg, Error, ResultExt};

#[derive(Debug)]
struct VaultContext {
    vault_path: String,
}

fn vault_context_from(args: &ArgMatches) -> Result<VaultContext, Error> {
    Ok(VaultContext {
        vault_path: args.value_of("config-file")
            .map(ToOwned::to_owned)
            .ok_or(err_msg("expected clap argument was unset"))?,
    })
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
            let context = vault_context_from(args)
                .context("context creation failed")
                .unwrap();
            println!("Parsed opts");
            println!("{:?}", context);
        }
        _ => {
            println!("{}", matches.usage());
        }
    }
}
