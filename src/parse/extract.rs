use atty;
use failure::Error;
use clap::ArgMatches;
use tools::merge::{Command, OutputMode};
use parse::util::optional_args_with_value;

use std::path::PathBuf;

pub fn context_from(args: &ArgMatches) -> Result<Vec<Command>, Error> {
    Ok({
        let mut has_seen_merge_stdin = false;
        let mut cmds = match (args.values_of_os("file"), args.indices_of("file")) {
            (Some(v), Some(i)) => v.map(|v| {
                if v == "-" {
                    has_seen_merge_stdin = true;
                    Command::MergeStdin
                } else {
                    Command::MergePath(PathBuf::from(v))
                }
            }).zip(i)
                .collect(),
            (None, None) => Vec::new(),
            _ => unreachable!("expecting clap to work"),
        };

        let select_cmds = optional_args_with_value(args, "pointer", |s| Command::SelectToBuffer(s.to_owned()));
        cmds.extend(select_cmds.into_iter());

        cmds.sort_by_key(|&(_, index)| index);
        let mut cmds: Vec<_> = cmds.into_iter().map(|(c, _)| c).collect();

        if let Ok(output_mode) = value_t!(args, "output", OutputMode) {
            cmds.insert(0, Command::SetOutputMode(output_mode));
        }

        if atty::isnt(atty::Stream::Stdin) && !has_seen_merge_stdin {
            let at_position = cmds.iter()
                .position(|cmd| match *cmd {
                    Command::MergePath(_) | Command::SelectToBuffer(_) => true,
                    _ => false,
                })
                .unwrap_or_else(|| cmds.len());
            cmds.insert(at_position, Command::MergeStdin)
        }
        cmds.push(Command::SerializeBuffer);

        if !cmds.iter().any(|c| match *c {
            Command::MergeStdin | Command::MergePath(_) => true,
            _ => false,
        }) {
            bail!("Please provide structured data from standard input or from a file.");
        }
        cmds
    })
}
