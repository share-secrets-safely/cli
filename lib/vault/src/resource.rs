use s3_types::VaultSpec;
use std::path::Path;
use failure::Error;

pub fn add(_vault_path: &Path, _specs: &[VaultSpec]) -> Result<String, Error> {
    Ok(String::new())
}
