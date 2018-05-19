#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate atty;
extern crate glob;
extern crate sheesy_tools as tools;
#[macro_use]
extern crate human_panic;

use clap::ArgMatches;

mod util;
mod cli;
mod parse;

use util::ok_or_exit;

fn main() {
    setup_panic!();
    let cli = cli::merge::new()
        .version(crate_version!())
        .author(crate_authors!())
        .name("syp");
    let matches: ArgMatches = cli.get_matches();
    ok_or_exit(parse::merge::execute(&matches))
}
