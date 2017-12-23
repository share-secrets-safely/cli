use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum VaultCommand {
    Init {
        gpg_key_ids: Vec<String>,
        gpg_keys_dir: PathBuf,
    },
    List,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VaultContext {
    pub vault_path: PathBuf,
    pub command: VaultCommand,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExtractionContext {
    pub file_path: PathBuf,
}
