use s3_types::VaultSpec;
use types::Vault;
use failure::Error;

pub fn add(_vault: Vault, _specs: &[VaultSpec]) -> Result<String, Error> {
    Ok(String::new())
}
