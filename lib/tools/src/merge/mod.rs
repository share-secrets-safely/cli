use failure::Error;

use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Command {
    MergePath(PathBuf),
}

pub fn merge(_output_mode: &str, _cmds: &[Command]) -> Result<(), Error> {
    Ok(())
}
