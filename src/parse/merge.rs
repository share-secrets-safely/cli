use atty;
use failure::Error;
use clap::ArgMatches;
use tools::merge::{Command, OutputMode};

use std::path::PathBuf;

use std::io::stdout;

pub fn context_from(args: &ArgMatches) -> Result<Vec<Command>, Error> {
    Ok({
        let mut cmds = match args.values_of_os("path") {
            Some(v) => v.map(|v| Command::MergePath(PathBuf::from(v))).collect(),
            None => Vec::new(),
        };
        let output_mode = value_t!(args, "output", OutputMode).expect("clap to work");

        if atty::isnt(atty::Stream::Stdin) {
            cmds.insert(0, Command::MergeStdin)
        }
        cmds.insert(0, Command::SetOutputMode(output_mode));
        cmds.push(Command::Serialize(Box::new(stdout())));
        cmds
    })
}
