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

use clap::ArgMatches;

mod dispatch;
mod util;
mod cli;
mod parse;

use util::ok_or_exit;

fn main() {
    let cli = cli::vault::new();
    let matches: ArgMatches = cli.get_matches();
    ok_or_exit(parse::vault::execute(&matches))
}
