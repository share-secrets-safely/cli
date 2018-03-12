use failure::Error;
use json;
use yaml;
use serde::Serialize;

use std::io;
mod types;
pub use self::types::*;

mod util;

pub fn merge(_cmds: &[Command]) -> Result<json::Value, Error> {
    Ok(json::Value::Null)
}

pub fn show<V, W>(output_mode: OutputMode, value: &V, ostream: W) -> Result<(), Error>
where
    V: Serialize,
    W: io::Write,
{
    match output_mode {
        OutputMode::Json => json::to_writer_pretty(ostream, value).map_err(Into::into),
        OutputMode::Yaml => yaml::to_writer(ostream, value).map_err(Into::into),
    }
}
