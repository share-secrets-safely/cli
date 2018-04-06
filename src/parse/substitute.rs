use failure::Error;
use clap::ArgMatches;
use tools::substitute::{Engine, Spec, StreamOrPath};
use itertools::Itertools;

use super::util::required_os_arg;
use std::ffi::OsString;
use tools::substitute::substitute;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Context {
    pub validate: bool,
    pub replacements: Vec<(String, String)>,
    pub separator: OsString,
    pub engine: Engine,
    pub data: StreamOrPath,
    pub specs: Vec<Spec>,
}

pub fn context_from(args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        replacements: {
            let replace_cmds = args.values_of("replace")
                .map_or_else(Vec::new, |v| v.map(|s| s.to_owned()).collect());
            if replace_cmds.len() % 2 != 0 {
                bail!("Please provide --replace-value arguments in pairs of two. First the value to find, second the one to replace it with");
            }
            replace_cmds.into_iter().tuples().collect()
        },
        separator: required_os_arg(args, "separator")?,
        engine: args.value_of("engine").expect("clap to work").parse()?,
        validate: args.is_present("validate"),
        data: args.value_of_os("data").map_or(StreamOrPath::Stream, Into::into),
        specs: match args.values_of("spec") {
            Some(v) => v.map(Spec::from).collect(),
            None => Vec::new(),
        },
    })
}

pub fn execute(args: &ArgMatches) -> Result<(), Error> {
    let context = context_from(args)?;
    substitute(
        context.engine,
        &context.data,
        &context.specs,
        &context.separator,
        context.validate,
        &context.replacements,
    )
}
