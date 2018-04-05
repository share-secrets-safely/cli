#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate atty;
extern crate conv;
extern crate glob;
extern crate gpgme;
extern crate itertools;
extern crate sheesy_tools as tools;
extern crate sheesy_vault as vault;

mod cli;
mod parse;
mod dispatch;

use clap::ArgMatches;
#[cfg(feature = "rest")]
use tools::{substitute::substitute};
use cli::CLI;

mod util;

use util::*;

fn main() {
    let cli = CLI::new();
    let appc = cli.app.clone();
    let matches: ArgMatches = cli.app.get_matches();

    let res = match matches.subcommand() {
        ("completions", Some(args)) => parse::completions::generate(appc, args),
        ("extract", Some(args)) => {
            parse::extract::execute(args)
        }
        ("process", Some(args)) => parse::merge::execute(args),
        ("substitute", Some(args)) => {
            let context = ok_or_exit(parse::substitute::context_from(args));
            substitute(
                context.engine,
                &context.data,
                &context.specs,
                &context.separator,
                context.validate,
                &context.replacements,
            )
        }
        ("vault", Some(args)) => parse::vault::execute(args),
        _ => panic!("Expected clap to prevent this"),
    };

    ok_or_exit(res);
}
