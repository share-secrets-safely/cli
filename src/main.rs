#[macro_use]
extern crate clap;
extern crate conv;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate sheesy_extract as extract;
extern crate sheesy_types;
extern crate sheesy_vault as vault;

mod cli;
mod parse;

use sheesy_types::print_causes;
use clap::ArgMatches;
use failure::Error;
use std::io::{stderr, stdout, Write};
use std::process;
use std::convert::Into;
use cli::CLI;
use parse::*;
use vault::error::{first_cause_of_type, DecryptionError};

fn add_vault_context<T>(r: Result<T, Error>) -> Result<T, Error> {
    r.map_err(|e| {
        let ctx = match first_cause_of_type::<DecryptionError>(&e) {
            Some(_err) => Some(format!(
                "Export your public key using '{} vault recipient init', then \
                 ask one of the existing recipients to import your public key \
                 using '{} vault recipients add <your-userid>.'",
                CLI::name(),
                CLI::name()
            )),
            None => None,
        };
        (e, ctx)
    }).map_err(|(e, msg)| match msg {
            Some(msg) => e.context(msg).into(),
            None => e,
        })
}

fn ok_or_exit<T, E>(r: Result<T, E>) -> T
where
    E: Into<Error>,
{
    match r {
        Ok(r) => r,
        Err(e) => {
            write!(stderr(), "error: ").ok();
            print_causes(e, stderr());
            process::exit(1);
        }
    }
}

fn usage_and_exit(args: &ArgMatches) -> ! {
    println!("{}", args.usage());
    process::exit(1)
}

fn main() {
    let cli = CLI::new();
    let appc = cli.app.clone();
    let matches: ArgMatches = cli.app.get_matches();

    let res = match matches.subcommand() {
        ("completions", Some(args)) => generate_completions(appc, args),
        ("vault", Some(args)) => {
            let mut context = ok_or_exit(vault_context_from(args));
            context = match args.subcommand() {
                ("recipients", Some(args)) => {
                    match args.subcommand() {
                        ("add", Some(args)) => ok_or_exit(vault_recipients_add(context, args)),
                        ("init", Some(args)) => ok_or_exit(vault_recipients_init(context, args)),
                        ("list", Some(args)) => ok_or_exit(vault_recipients_list(context, args)),
                        _ => ok_or_exit(vault_recipients_list(context, args)),
                    }
                }
                ("init", Some(args)) => ok_or_exit(vault_init_from(context, args)),
                ("add", Some(args)) => ok_or_exit(vault_resource_add(context, args)),
                ("remove", Some(args)) => ok_or_exit(vault_resource_remove(context, args)),
                ("show", Some(args)) => ok_or_exit(vault_resource_show(context, args)),
                ("edit", Some(args)) => ok_or_exit(vault_resource_edit(context, args)),
                ("list", Some(args)) => ok_or_exit(vault_resource_list(context, args)),
                _ => context,
            };
            let sout = stdout();
            let mut lock = sout.lock();
            add_vault_context(vault::do_it(context, &mut lock))
        }
        ("extract", Some(args)) => {
            let context = ok_or_exit(extraction_context_from(args));
            extract::do_it(context)
        }
        _ => usage_and_exit(&matches),
    };

    ok_or_exit(res);
}
