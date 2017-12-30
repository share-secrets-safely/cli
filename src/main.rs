#[macro_use]
extern crate clap;
extern crate conv;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate s3_extract as extract;
extern crate s3_types;
extern crate s3_vault as vault;

mod cli;
mod parse;

use clap::ArgMatches;
use failure::Error;
use std::io::{stderr, stdout, Write};
use std::process;
use std::convert::Into;
use cli::CLI;
use parse::*;
use vault::error::{first_cause_of_type, DecryptError};

fn add_vault_context<T>(r: Result<T, Error>) -> Result<T, Error> {
    r.map_err(|e| {
        let ctx_msg = match first_cause_of_type::<DecryptError>(&e) {
            Some(_err) => Some(format!(
                "Ask one of the existing recipients to import your public key \
                 using '{} vault recipients add <your-userid>.'",
                CLI::name()
            )),
            None => None,
        };
        if let Some(msg) = ctx_msg {
            e.context(msg).into()
        } else {
            e.into()
        }
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
                ("recipients", Some(args)) => match args.subcommand() {
                    ("add", Some(args)) => ok_or_exit(vault_recipients_add(context, args)),
                    _ => usage_and_exit(&matches),
                },
                ("init", Some(args)) => ok_or_exit(vault_init_from(context, args)),
                ("add", Some(args)) => ok_or_exit(vault_resource_add_from(context, args)),
                ("show", Some(args)) => ok_or_exit(vault_resource_show(context, args)),
                ("edit", Some(args)) => ok_or_exit(vault_resource_edit(context, args)),
                _ => context,
            };
            let sout = stdout();
            let mut lock = sout.lock();
            add_vault_context((vault::do_it(context, &mut lock)))
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
