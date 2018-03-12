use failure::Error;

use std::path::PathBuf;
use std::str::FromStr;

use std::borrow::Cow;
use treediff::tools::MutableFilter;
use std::fmt;

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

#[derive(Default)]
pub struct NeverDrop {
    pub clashed_keys: Vec<String>,
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
        _new: &'a V,
        _self: &mut V,
    ) -> Option<Cow<'a, V>> {
        self.clashed_keys.push(
            keys.iter()
                .map(|k| format!("{}", k))
                .collect::<Vec<_>>()
                .join("."),
        );
        Some(Cow::Borrowed(old))
    }
}
