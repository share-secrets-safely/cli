use base::Vault;
use failure::Error;
use std::io::Write;
use std::path::Path;

impl Vault {
    pub fn add_partition(&self, path: &Path, name: Option<&str>, output: &mut Write) -> Result<(), Error> {
        Ok(())
    }
}
