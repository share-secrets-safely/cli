use std::path::PathBuf;
use vault::{CreateMode, SigningMode, TrustModel, VaultSpec};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Command {
    ResourceEdit {
        editor: PathBuf,
        try_encrypt: bool,
        spec: PathBuf,
        mode: CreateMode,
    },
    ResourceShow {
        spec: PathBuf,
    },
    ResourceAdd {
        specs: Vec<VaultSpec>,
    },
    ResourceRemove {
        specs: Vec<PathBuf>,
    },
    Init {
        name: Option<String>,
        trust_model: Option<TrustModel>,
        auto_import: Option<bool>,
        gpg_key_ids: Vec<String>,
        gpg_keys_dir: PathBuf,
        secrets: PathBuf,
        recipients_file: PathBuf,
    },
    RecipientsList,
    RecipientsInit {
        gpg_key_ids: Vec<String>,
    },
    RecipientsRemove {
        gpg_key_ids: Vec<String>,
        partitions: Vec<String>,
    },
    RecipientsAdd {
        partitions: Vec<String>,
        gpg_key_ids: Vec<String>,
        signing_key_id: Option<String>,
        sign: SigningMode,
    },
    PartitionsRemove {
        selector: String,
    },
    PartitionsAdd {
        recipients_file: Option<PathBuf>,
        gpg_key_ids: Vec<String>,
        name: Option<String>,
        path: PathBuf,
    },
    List,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Context {
    pub vault_path: PathBuf,
    pub vault_selector: String,
    pub command: Command,
}
