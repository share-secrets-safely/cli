use failure::{Error, ResultExt};
use json;
use yaml;
use serde::Serialize;

mod types;
pub use self::types::*;
use std::io::{self, stdin};
use std::fs::File;
use std::env::vars;
use treediff::{diff, tools};

mod util;

fn validate(cmds: &[Command]) -> Result<(), Error> {
    let num_merge_stdin_cmds = cmds.iter()
        .filter(|c| if let &Command::MergeStdin = *c { true } else { false })
        .count();
    if num_merge_stdin_cmds > 1 {
        bail!(
            "Cannot read from stdin more than once, found {} invocations",
            num_merge_stdin_cmds
        );
    }
    Ok(())
}

pub fn reduce(cmds: Vec<Command>, initial_state: Option<State>, mut output: &mut io::Write) -> Result<State, Error> {
    validate(&cmds)?;

    use self::Command::*;
    let mut state = initial_state.unwrap_or_else(State::default);

    for cmd in cmds {
        match cmd {
            InsertNextMergeAt(at) => {
                state.insert_next_at = Some(at);
            }
            SetMergeMode(mode) => {
                state.merge_mode = mode;
            }
            MergeStdin => {
                let value_to_merge = util::de_json_or_yaml_document_support(stdin(), &state)?;
                state = merge(value_to_merge, state)?;
            }
            MergeEnvironment(pattern) => {
                let mut map = vars().filter(|&(ref var, _)| pattern.matches(var)).fold(
                    json::Map::new(),
                    |mut m, (var, value)| {
                        m.insert(var, json::from_str(&value).unwrap_or_else(|_| json::Value::from(value)));
                        m
                    },
                );
                state = merge(json::Value::from(map), state)?;
            }
            MergePath(path) => {
                let reader =
                    File::open(&path).context(format!("Failed to open file at '{}' for reading", path.display()))?;
                let value_to_merge = util::de_json_or_yaml_document_support(reader, &state)?;
                state = merge(value_to_merge, state)?;
            }
            SetOutputMode(mode) => {
                state.output_mode = mode;
            }
            Serialize => show(&state.output_mode, &state.value, &mut output)?,
        }
    }
    if let Some(pos) = state.insert_next_at.take() {
        bail!("The insertion position named '{}' was not consumed", pos)
    }
    Ok(state)
}

fn show<V, W>(output_mode: &OutputMode, value: &V, ostream: W) -> Result<(), Error>
where
    V: Serialize,
    W: io::Write,
{
    match *output_mode {
        OutputMode::Json => json::to_writer_pretty(ostream, value).map_err(Into::into),
        OutputMode::Yaml => yaml::to_writer(ostream, value).map_err(Into::into),
    }
}

fn insert_json_at(pointer: Option<String>, v: json::Value) -> Result<json::Value, Error> {
    Ok(match pointer {
        Some(mut pointer) => {
            if pointer.find('/').is_none() {
                pointer = pointer.replace('.', "/");
            }
            let mut current = v;
            for elm in pointer.rsplit('/') {
                let index: Result<usize, _> = elm.parse();
                match index {
                    Ok(index) => {
                        let mut a = vec![json::Value::Null; index + 1];
                        a[index] = current;
                        current = json::Value::from(a);
                    }
                    Err(_) => {
                        let mut map = json::Map::new();
                        map.insert(elm.to_owned(), current);
                        current = json::Value::from(map)
                    }
                }
            }
            current
        }
        None => v,
    })
}

fn merge(src: json::Value, mut state: State) -> Result<State, Error> {
    match state.value {
        None => {
            let src = insert_json_at(state.insert_next_at.take(), src)?;
            state.value = Some(src);
            Ok(state)
        }
        Some(existing_value) => {
            let mut m = tools::Merger::with_filter(existing_value.clone(), NeverDrop::with_mode(&state.merge_mode));
            let src = insert_json_at(state.insert_next_at.take(), src)?;
            diff(&existing_value, &src, &mut m);

            if m.filter().clashed_keys.len() > 0 {
                Err(format_err!("{}", m.filter())
                    .context("The merge failed due to conflicts")
                    .into())
            } else {
                state.value = Some(m.into_inner());
                Ok(state)
            }
        }
    }
}
