use failure::Error;
use json;

use std::path::PathBuf;
use std::str::FromStr;

use std::borrow::Cow;
use treediff::tools::MutableFilter;
use std::fmt;

pub struct State {
    pub output_mode: OutputMode,
    pub merge_mode: MergeMode,
    pub value: Option<json::Value>,
}

impl Default for State {
    fn default() -> Self {
        State {
            output_mode: OutputMode::Json,
            merge_mode: MergeMode::NoOverwrite,
            value: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MergeMode {
    Overwrite,
    NoOverwrite,
}

#[derive(Debug)]
pub enum Command {
    MergeStdin,
    MergePath(PathBuf),
    SetMergeMode(MergeMode),
    Serialize,
    SetOutputMode(OutputMode),
}

#[derive(Debug)]
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

pub struct NeverDrop {
    pub mode: MergeMode,
    pub clashed_keys: Vec<String>,
}

impl NeverDrop {
    pub fn with_mode(mode: &MergeMode) -> Self {
        NeverDrop {
            mode: mode.clone(),
            clashed_keys: Vec::new(),
        }
    }
}

impl fmt::Display for NeverDrop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Refusing to merge due to the following clashing keys:")?;
        for key in &self.clashed_keys {
            writeln!(f, "{}\n", key)?;
        }
        Ok(())
    }
}

impl MutableFilter for NeverDrop {
    fn resolve_removal<'a, K: fmt::Display, V: Clone>(
        &mut self,
        _keys: &[K],
        removed: &'a V,
        _self: &mut V,
    ) -> Option<Cow<'a, V>> {
        Some(Cow::Borrowed(removed))
    }

    fn resolve_conflict<'a, K: fmt::Display, V: Clone>(
        &mut self,
        keys: &[K],
        old: &'a V,
        new: &'a V,
        _self: &mut V,
    ) -> Option<Cow<'a, V>> {
        match self.mode {
            MergeMode::NoOverwrite => {
                self.clashed_keys
                    .push(keys.iter().map(|k| format!("{}", k)).collect::<Vec<_>>().join("."));
                Some(Cow::Borrowed(old))
            }
            MergeMode::Overwrite => Some(Cow::Borrowed(new)),
        }
    }
}
