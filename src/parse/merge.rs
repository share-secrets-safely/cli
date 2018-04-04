use atty;
use glob;
use failure::Error;
use clap::ArgMatches;
use tools::merge::{reduce, Command, MergeMode, OutputMode};
use parse::util::optional_args_with_value;

use std::path::PathBuf;
use std::io::stdout;

pub fn execute(args: &ArgMatches) -> Result<(), Error> {
    let cmds = context_from(args)?;

    let sout = stdout();
    let mut lock = sout.lock();
    reduce(cmds, None, &mut lock).map(|_| ())
}

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
        let mut has_seen_merge_stdin = false;
        let mut cmds = match (args.values_of_os("path"), args.indices_of("path")) {
            (Some(v), Some(i)) => v.map(|v| {
                if v == "-" {
                    has_seen_merge_stdin = true;
                    Command::MergeStdin
                } else {
                    match v.to_str().map(|v| (v, v.find('='))) {
                        Some((v, Some(idx))) => Command::MergeValue(v[..idx].to_owned(), v[idx + 1..].to_owned()),
                        _ => Command::MergePath(PathBuf::from(v)),
                    }
                }
            }).zip(i)
                .collect(),
            (None, None) => Vec::new(),
            _ => unreachable!("expecting clap to work"),
        };
        cmds.extend(optional_args_without_value(args, "overwrite", || {
            Command::SetMergeMode(MergeMode::Overwrite)
        }));
        cmds.extend(optional_args_without_value(args, "no-overwrite", || {
            Command::SetMergeMode(MergeMode::NoOverwrite)
        }));

        let env_cmds = optional_args_with_value(args, "environment", |s| {
            Command::MergeEnvironment(glob::Pattern::new(s).expect("clap to work"))
        });
        cmds.extend(env_cmds.into_iter());

        let at_cmds = optional_args_with_value(args, "at", |s| Command::InsertNextMergeAt(s.to_owned()));
        cmds.extend(at_cmds.into_iter());
        let select_cmds = optional_args_with_value(args, "select", |s| Command::SelectNextMergeAt(s.to_owned()));
        cmds.extend(select_cmds.into_iter());

        cmds.sort_by_key(|&(_, index)| index);
        let mut cmds: Vec<_> = cmds.into_iter().map(|(c, _)| c).collect();

        let output_mode = value_t!(args, "output", OutputMode).expect("clap to work");
        cmds.insert(0, Command::SetOutputMode(output_mode));

        if atty::isnt(atty::Stream::Stdin) && !has_seen_merge_stdin {
            let at_position = cmds.iter()
                .position(|cmd| match *cmd {
                    Command::MergePath(_) | Command::MergeValue(_, _) | Command::MergeEnvironment(_) => true,
                    _ => false,
                })
                .unwrap_or_else(|| cmds.len());
            cmds.insert(at_position, Command::MergeStdin)
        }
        cmds.push(Command::Serialize);

        if !cmds.iter().any(|c| match *c {
            Command::MergeStdin | Command::MergeValue(_, _) | Command::MergePath(_) | Command::MergeEnvironment(_) => {
                true
            }
            _ => false,
        }) {
            bail!("Please provide structured data from standard input or from a file.");
        }
        cmds
    })
}
