use atty;
use failure::Error;
use clap::ArgMatches;
use tools::merge::Command;

use std::path::PathBuf;
use parse::util::required_arg;

#[derive(Debug)]
pub struct Context {
    pub cmds: Vec<Command>,
    pub mode: String,
}

pub fn context_from(args: &ArgMatches) -> Result<Context, Error> {
    Ok(Context {
        mode: required_arg(args, "output")?,
        cmds: {
            let mut cmds = match args.values_of_os("path") {
                Some(v) => v.map(|v| Command::MergePath(PathBuf::from(v))).collect(),
                None => Vec::new(),
            };
            if atty::isnt(atty::Stream::Stdin) {
                cmds.insert(0, Command::MergeStdin)
            }
            cmds
        },
    })
}
