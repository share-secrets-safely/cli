#[derive(Debug, Clone)]
pub enum VaultCommand {
    Init { gpg_key_ids: Vec<String> },
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
