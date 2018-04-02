#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[cfg(feature = "vault")]
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
