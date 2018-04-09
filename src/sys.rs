#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate itertools;
extern crate sheesy_tools as tools;

use clap::ArgMatches;

mod util;
mod cli;
mod parse;

use util::ok_or_exit;

fn main() {
    let cli = cli::substitute::new()
        .version(crate_version!())
        .author(crate_authors!())
        .name("sys");
    let matches: ArgMatches = cli.get_matches();
    ok_or_exit(parse::substitute::execute(&matches))
}
