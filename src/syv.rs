#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate conv;
extern crate gpgme;
extern crate sheesy_vault as vault;

use clap::ArgMatches;

pub mod cli;
pub mod dispatch;
pub mod parse;
pub mod util;

use util::ok_or_exit;

fn main() {
    let cli = cli::vault::new()
        .version(crate_version!())
        .author(crate_authors!())
        .name("syv");
    let matches: ArgMatches = cli.get_matches();
    ok_or_exit(parse::vault::execute(&matches))
}
