use failure::Error;
use clap::ArgMatches;
use conv::TryFrom;
use substitute::Spec;

use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Context {
    pub data: Option<PathBuf>,
    pub specs: Vec<Spec>,
}

pub fn context_from(args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        data: args.value_of_os("data").map(Into::into),
        specs: match args.values_of("spec") {
            Some(v) => v.map(Spec::try_from).collect::<Result<_, _>>()?,
            None => Vec::new(),
        },
    })
}
