use std::path::PathBuf;
use spec::VaultSpec;
use std::io::Write;
use failure::Error;

pub fn print_causes<E, W>(e: E, mut w: W)
where
    E: Into<Error>,
    W: Write,
{
    let e = e.into();
    let causes = e.causes().collect::<Vec<_>>();
    let num_causes = causes.len();
    for (index, cause) in causes.iter().enumerate() {
        if index == 0 {
            writeln!(w, "{}", cause).ok();
            if num_causes > 1 {
                writeln!(w, "Caused by: ").ok();
            }
        } else {
            writeln!(w, " {}: {}", num_causes - index, cause).ok();
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum VaultCommand {
    ResourceEdit {
        editor: PathBuf,
        spec: PathBuf,
        mode: CreateMode,
    },
    ResourceShow { spec: PathBuf },
    ResourceAdd { specs: Vec<VaultSpec> },
    ResourceRemove { specs: Vec<VaultSpec> },
    Init {
        gpg_key_ids: Vec<String>,
        gpg_keys_dir: PathBuf,
        secrets: PathBuf,
        recipients_file: PathBuf,
    },
    RecipientsList,
    RecipientsInit { gpg_key_ids: Vec<String> },
    RecipientsRemove { gpg_key_ids: Vec<String> },
    RecipientsAdd {
        gpg_key_ids: Vec<String>,
        signing_key_id: Option<String>,
        sign: SigningMode,
    },
    List,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Destination {
    ReolveAndAppendGpg,
    Unchanged,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WriteMode {
    AllowOverwrite,
    RefuseOverwrite,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SigningMode {
    None,
    Public,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CreateMode {
    Create,
    NoCreate,
}

impl WriteMode {
    pub fn refuse_overwrite(self) -> bool {
        match self {
            WriteMode::AllowOverwrite => false,
            WriteMode::RefuseOverwrite => true,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VaultContext {
    pub vault_path: PathBuf,
    pub vault_id: String,
    pub command: VaultCommand,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExtractionContext {
    pub file_path: PathBuf,
}
