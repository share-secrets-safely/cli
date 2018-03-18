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
        .filter(|c| if let Command::MergeStdin = **c { true } else { false })
        .count();
    if num_merge_stdin_cmds > 1 {
        bail!(
            "Cannot read from stdin more than once, found {} invocations",
            num_merge_stdin_cmds
        );
    }
    Ok(())
}

fn to_json(s: String, state: &State) -> json::Value {
    let mut reader = io::Cursor::new(s);
    util::de_json_or_yaml_document_support(&mut reader, state)
        .unwrap_or_else(|_| json::Value::from(reader.into_inner()))
}

pub fn reduce(cmds: Vec<Command>, initial_state: Option<State>, mut output: &mut io::Write) -> Result<State, Error> {
    validate(&cmds)?;

    use self::Command::*;
    let mut state = initial_state.unwrap_or_else(State::default);

    for cmd in cmds {
        match cmd {
            SelectNextMergeAt(at) => {
                state.select_next_at = Some(at);
            }
            InsertNextMergeAt(at) => {
                state.insert_next_at = Some(at);
            }
            SetMergeMode(mode) => {
                state.merge_mode = mode;
            }
            MergeValue(pointer, value) => {
                let value_to_merge = to_json(value, &state);
                let prev_insert_next_at = state.insert_next_at;
                state.insert_next_at = Some(pointer);

                state = merge(value_to_merge, state)?;

                state.insert_next_at = prev_insert_next_at;
            }
            MergeStdin => {
                let value_to_merge = util::de_json_or_yaml_document_support(stdin(), &state)?;
                state = merge(value_to_merge, state)?;
            }
            MergeEnvironment(pattern) => {
                let mut map = vars().filter(|&(ref var, _)| pattern.matches(var)).fold(
                    json::Map::new(),
                    |mut m, (var, value)| {
                        m.insert(var, to_json(value, &state));
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
            Serialize => {
                state.value = match state.value {
                    Some(value) => Some(apply_transforms(
                        value,
                        state.insert_next_at.take(),
                        state.select_next_at.take(),
                    )?),
                    None => None,
                };

                show(&state.output_mode, &state.value, &mut output)?
            }
        }
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

fn into_pointer(p: String) -> String {
    let mut p = if p.find('/').is_none() { p.replace('.', "/") } else { p };
    if !p.starts_with('/') {
        p.insert(0, '/');
    }
    p
}

fn select_json_at(pointer: Option<String>, v: json::Value) -> Result<json::Value, Error> {
    match pointer {
        Some(mut pointer) => {
            pointer = into_pointer(pointer);
            v.pointer(&pointer)
                .map(|v| v.to_owned())
                .ok_or_else(|| format_err!("No value at pointer '{}'", pointer))
        }
        None => Ok(v),
    }
}

fn insert_json_at(pointer: Option<String>, v: json::Value) -> Result<json::Value, Error> {
    Ok(match pointer {
        Some(mut pointer) => {
            pointer = into_pointer(pointer);
            let mut current = v;
            for elm in pointer.rsplit('/').filter(|s| !s.is_empty()) {
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

fn apply_transforms(
    src: json::Value,
    insert_at: Option<String>,
    select_at: Option<String>,
) -> Result<json::Value, Error> {
    select_json_at(select_at, src).and_then(|src| insert_json_at(insert_at, src))
}

fn merge(src: json::Value, mut state: State) -> Result<State, Error> {
    let src = apply_transforms(src, state.insert_next_at.take(), state.select_next_at.take())?;

    match state.value {
        None => {
            state.value = Some(src);
            Ok(state)
        }
        Some(existing_value) => {
            let mut m = tools::Merger::with_filter(existing_value.clone(), NeverDrop::with_mode(&state.merge_mode));
            diff(&existing_value, &src, &mut m);

            if !m.filter().clashed_keys.is_empty() {
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
