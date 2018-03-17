use atty;
use failure::Error;
use clap::ArgMatches;
use tools::merge::{Command, MergeMode, OutputMode};

use std::path::PathBuf;

fn optional_args_without_value<F>(args: &ArgMatches, name: &'static str, into: F) -> Vec<(Command, usize)>
where
    F: Fn() -> Command,
{
    match args.indices_of(name) {
        Some(v) => v.map(|i| (into(), i)).collect(),
        None => Vec::new(),
    }
}

pub fn context_from(args: &ArgMatches) -> Result<Vec<Command>, Error> {
    Ok({
        let mut cmds = match (args.values_of_os("path"), args.indices_of("path")) {
            (Some(v), Some(i)) => v.map(|v| Command::MergePath(PathBuf::from(v))).zip(i).collect(),
            (None, None) => Vec::new(),
            _ => unreachable!("expecting clap to work"),
        };
        cmds.extend(optional_args_without_value(args, "overwrite", || {
            Command::SetMergeMode(MergeMode::Overwrite)
        }));
        cmds.extend(optional_args_without_value(args, "no-overwrite", || {
            Command::SetMergeMode(MergeMode::NoOverwrite)
        }));

        cmds.sort_by_key(|&(_, index)| index);
        let mut cmds: Vec<_> = cmds.into_iter().map(|(c, _)| c).collect();

        let output_mode = value_t!(args, "output", OutputMode).expect("clap to work");
        cmds.insert(0, Command::SetOutputMode(output_mode));

        if atty::isnt(atty::Stream::Stdin) {
            let at_position = cmds.iter()
                .position(|cmd| match *cmd {
                    Command::MergePath(_) => true,
                    _ => false,
                })
                .unwrap_or(cmds.len());
            cmds.insert(at_position, Command::MergeStdin)
        }
        cmds.push(Command::Serialize);

        if !cmds.iter().any(|c| match *c {
            Command::MergeStdin => true,
            Command::MergePath(_) => true,
            _ => false,
        }) {
            bail!("Please provide structured data from standard input or from a file.");
        }
        cmds
    })
}
