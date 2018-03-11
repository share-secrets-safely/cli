use failure::Error;
use clap::ArgMatches;
use tools::substitute::{Spec, StreamOrPath};

use super::util::required_os_arg;
use std::ffi::OsString;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Context {
    pub separator: OsString,
    pub data: StreamOrPath,
    pub specs: Vec<Spec>,
}

pub fn context_from(args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        separator: required_os_arg(args, "separator")?,
        data: args.value_of_os("data")
            .map_or(StreamOrPath::Stream, Into::into),
        specs: match args.values_of("spec") {
            Some(v) => v.map(Spec::from).collect(),
            None => Vec::new(),
        },
    })
}
