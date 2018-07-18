#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate atty;
extern crate glob;
extern crate sheesy_tools as tools;

use clap::ArgMatches;

mod cli;
mod parse;
mod util;

use util::ok_or_exit;

fn main() {
    let cli = cli::merge::new()
        .version(crate_version!())
        .author(crate_authors!())
        .name("syp");
    let matches: ArgMatches = cli.get_matches();
    ok_or_exit(parse::merge::execute(&matches))
}
