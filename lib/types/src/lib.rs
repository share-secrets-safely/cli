#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum VaultCommand {
    Init { gpg_key_ids: Vec<String> },
    List,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VaultContext {
    pub vault_path: String,
    pub command: VaultCommand,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExtractionContext {
    pub file_path: String,
}
