use failure::Error;
use json;

use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Command {
    MergeStdin,
    MergePath(PathBuf),
}

pub enum OutputMode {
    Json,
    Yaml,
}

impl FromStr for OutputMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "json" => OutputMode::Json,
            "yaml" => OutputMode::Yaml,
            _ => bail!("Not a valid output mode: '{}'", s),
        })
    }
}

pub struct State {
    value: json::Value,
}
