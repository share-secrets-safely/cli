use failure::{Error, ResultExt};
use json;
use yaml;
use serde::Serialize;

use std::io;
mod types;
pub use self::types::*;
use std::io::stdin;
use std::fs::File;

mod util;

pub fn reduce(cmds: Vec<Command>, initial_state: Option<State>) -> Result<State, Error> {
    use self::Command::*;
    let mut state = initial_state.unwrap_or_else(State::default);

    for cmd in cmds {
        match cmd {
            MergeStdin => {
                let value_to_merge = util::de_json_or_yaml_document_support(stdin())?;
                state.value = value_to_merge
                // TODO: merge it into what exists
            }
            MergePath(path) => {
                let reader =
                    File::open(&path).context(format!("Failed to open file at '{}' for reading", path.display()))?;
                let value_to_merge = util::de_json_or_yaml_document_support(reader)?;
                state.value = value_to_merge;
                // TODO: merge it into what exists
            }
            SetOutputMode(mode) => {
                state.output_mode = mode;
            }
            Serialize(write) => show(&state.output_mode, &state.value, write)?,
        }
    }
    Ok(state)
}

pub fn show<V, W>(output_mode: &OutputMode, value: &V, ostream: W) -> Result<(), Error>
where
    V: Serialize,
    W: io::Write,
{
    match *output_mode {
        OutputMode::Json => json::to_writer_pretty(ostream, value).map_err(Into::into),
        OutputMode::Yaml => yaml::to_writer(ostream, value).map_err(Into::into),
    }
}
