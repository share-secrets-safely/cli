use failure::Error;
use clap::ArgMatches;
use tools::merge::Command;

use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Context {
    pub cmds: Vec<Command>,
    pub mode: String,
}

pub fn context_from(args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        mode: args.value_of("output")
            .map(ToOwned::to_owned)
            .expect("clap to work"),
        cmds: match args.values_of_os("path") {
            Some(v) => v.map(|v| Command::MergePath(PathBuf::from(v))).collect(),
            None => Vec::new(),
        },
    })
}
