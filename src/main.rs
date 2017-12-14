#[macro_use]
extern crate clap;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "s3", about = "The share-secrets-safely command-line interface.")]
struct Opts {
    #[structopt(short = "v", long = "vault",
                help = "The configuration file describing the vault.")]
    vault: String,
}

fn main() {
    let app: clap::App = Opts::clap().version(crate_version!());
    let opts = Opts::from_clap(app.get_matches());

    println!("Parsed opts");
    println!("{:?}", opts)
}
