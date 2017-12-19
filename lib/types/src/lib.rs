use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum VaultCommand {
    Init {
        gpg_keyfile_path: Option<PathBuf>,
        gpg_key_id: Option<String>,
    },
    List,
}

#[derive(Debug, Clone)]
pub struct VaultContext {
    pub vault_path: String,
    pub command: VaultCommand,
}

#[derive(Debug, Clone)]
pub struct ExtractionContext {
    pub file_path: String,
}
