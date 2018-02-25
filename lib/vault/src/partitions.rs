use base::Vault;
use failure::Error;
use std::io::Write;
use std::path::Path;

impl Vault {
    pub fn add_partition(&self, _path: &Path, _name: Option<&str>, _output: &mut Write) -> Result<(), Error> {
        Ok(())
    }
}
