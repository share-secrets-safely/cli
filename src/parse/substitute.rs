use failure::Error;
use clap::ArgMatches;
use substitute::Spec;

use super::util::required_os_arg;
use std::path::PathBuf;
use std::ffi::OsString;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Context {
    pub separator: OsString,
    pub data: Option<PathBuf>,
    pub specs: Vec<Spec>,
}

pub fn context_from(args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        separator: required_os_arg(args, "separator")?,
        data: args.value_of_os("data").map(Into::into),
        specs: match args.values_of("spec") {
            Some(v) => v.map(Spec::from).collect(),
            None => Vec::new(),
        },
    })
}
