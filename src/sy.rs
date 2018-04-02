extern crate atty;
#[macro_use]
extern crate clap;
extern crate conv;
#[macro_use]
extern crate failure;
extern crate gpgme;
#[macro_use]
extern crate lazy_static;
extern crate glob;
extern crate itertools;
extern crate sheesy_tools as tools;
extern crate sheesy_vault as vault;

mod cli;
mod parse;
mod dispatch;

use clap::ArgMatches;
use tools::substitute::substitute;
use tools::merge::reduce;
use std::io::stdout;
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
            let cmds = ok_or_exit(parse::extract::context_from(args));

            let sout = stdout();
            let mut lock = sout.lock();
            reduce(cmds, None, &mut lock).map(|_| ())
        }
        ("process", Some(args)) => {
            let cmds = ok_or_exit(parse::merge::context_from(args));

            let sout = stdout();
            let mut lock = sout.lock();
            reduce(cmds, None, &mut lock).map(|_| ())
        }
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
