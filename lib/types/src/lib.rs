extern crate conv;
#[macro_use]
extern crate failure;

use conv::TryFrom;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{stdin, Read};

use failure::{Error, ResultExt};
use std::path::{Path, PathBuf};
use std::path::Component;

pub fn gpg_output_filename(path: &Path) -> Result<PathBuf, Error> {
    let file_name = path.file_name().ok_or_else(|| {
        format_err!("'{}' does not have a filename", path.display())
    })?;
    Ok(
        path.parent()
            .expect("path with filename to have a root")
            .join(format!(
                "{}.gpg",
                file_name.to_str().expect(
                    "filename to be decodeable with UTF8",
                )
            )),
    )
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
    Init {
        gpg_key_ids: Vec<String>,
        gpg_keys_dir: PathBuf,
        secrets: PathBuf,
        recipients_file: PathBuf,
    },
    RecipientsList,
    RecipientsInit { gpg_key_ids: Vec<String> },
    RecipientsAdd {
        gpg_key_ids: Vec<String>,
        verified: bool,
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
pub struct VaultSpec {
    pub src: Option<PathBuf>,
    pub dst: PathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VaultSpecError(String);

impl fmt::Display for VaultSpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for VaultSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let empty = PathBuf::new();
        write!(
            f,
            "{}:{}",
            self.src.as_ref().unwrap_or_else(|| &empty).display(),
            self.dst.display()
        )
    }
}

impl ::std::error::Error for VaultSpecError {
    fn description(&self) -> &str {
        "The vault spec was invalid"
    }
}

impl VaultSpec {
    pub fn source(&self) -> Option<&Path> {
        self.src.as_ref().map(|s| s.as_ref())
    }

    pub fn destination(&self) -> &Path {
        &self.dst
    }

    pub fn open_output_in(&self, root: &Path, mode: WriteMode, dst_mode: Destination) -> Result<File, Error> {
        let output_file = match dst_mode {
            Destination::ReolveAndAppendGpg => root.join(gpg_output_filename(&self.dst)?),
            Destination::Unchanged => self.dst.to_owned(),
        };
        if mode.refuse_overwrite() && output_file.exists() {
            return Err(format_err!(
                "Refusing to overwrite existing file at '{}'",
                output_file.display()
            ));
        }
        Ok(OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&output_file)
            .context(format!(
                "Could not open destination file at '{}' for writing.",
                output_file.display()
            ))?)
    }

    pub fn open_input(&self) -> Result<Box<Read>, Error> {
        Ok(match self.src {
            Some(ref p) => Box::new(File::open(p).context(format!(
                "Could not open input file at '{}'",
                p.display()
            ))?),
            None => Box::new(stdin()),
        })
    }
}

impl<'a> TryFrom<&'a str> for VaultSpec {
    type Err = VaultSpecError;

    fn try_from(input: &'a str) -> Result<Self, Self::Err> {
        const SEPARATOR: char = ':';
        fn has_parent_component(p: &Path) -> bool {
            p.components().any(|c| match c {
                Component::ParentDir => true,
                _ => false,
            })
        }
        let validate = |src: &'a str| {
            Ok(if src.is_empty() {
                None
            } else {
                Some(PathBuf::from(src))
            })
        };
        let validate_dst = |p: PathBuf| if p.is_absolute() {
            Err(VaultSpecError(format!(
                "'{}' must not have an absolute destination.",
                input
            )))
        } else {
            Ok(p)
        };

        if input.is_empty() {
            return Err(VaultSpecError("An empty spec is invalid.".into()));
        }
        let mut splits = input.splitn(2, SEPARATOR);
        Ok(match (splits.next(), splits.next()) {
            (Some(src), None) => VaultSpec {
                src: validate(src)?,
                dst: {
                    let dst = validate_dst(PathBuf::from(src)).map_err(|mut e| {
                        e.0.push_str(" Try specifying the destination explicitly.");
                        e
                    })?;
                    if has_parent_component(&dst) {
                        return Err(VaultSpecError(format!(
                            "Relative parent directories in source '{}' need the destination set explicitly.",
                            src
                        )));
                    };
                    dst
                },
            },
            (Some(src), Some(dst)) => VaultSpec {
                src: validate(src)?,
                dst: PathBuf::from(if dst.is_empty() {
                    if src.is_empty() {
                        return Err(VaultSpecError(
                            format!("'{}' does not contain a destination.", input),
                        ));
                    }
                    src
                } else if dst.contains(SEPARATOR) {
                    return Err(VaultSpecError(
                        format!("'{}' must not contain more than one colon.", input),
                    ));
                } else {
                    dst
                }),
            },
            _ => unreachable!(),
        })
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

#[cfg(test)]
mod tests_gpg_output_filename {
    use super::gpg_output_filename;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn it_appends_the_gpg_suffix_to_file_names() {
        assert_eq!(
            gpg_output_filename(Path::new("a/file")).unwrap(),
            PathBuf::from("a/file.gpg")
        )
    }

    #[test]
    fn it_appends_the_gpg_suffix_to_file_names_with_extension() {
        assert_eq!(
            gpg_output_filename(Path::new("a/file.ext")).unwrap(),
            PathBuf::from("a/file.ext.gpg")
        )
    }
}

#[cfg(test)]
mod tests_vault_spec {
    use super::*;

    #[test]
    fn it_cannot_have_just_a_multiple_separator() {
        let invalid = ":::";
        assert_eq!(
            VaultSpec::try_from(invalid),
            Err(VaultSpecError(format!(
                "'{}' must not contain more than one colon.",
                invalid
            )))
        )
    }

    #[test]
    fn it_cannot_have_just_a_separator() {
        let invalid = ":";
        assert_eq!(
            VaultSpec::try_from(invalid),
            Err(VaultSpecError(
                format!("'{}' does not contain a destination.", invalid),
            ))
        )
    }

    #[test]
    fn it_cannot_have_more_than_one_separator() {
        let invalid = "relative/path:other/path:yet/another/path";
        assert_eq!(
            VaultSpec::try_from(invalid),
            Err(VaultSpecError(format!(
                "'{}' must not contain more than one colon.",
                invalid
            )))
        )
    }

    #[test]
    fn it_is_ok_to_not_specify_a_source_to_signal_stdin() {
        assert_eq!(
            VaultSpec::try_from(":other/path"),
            Ok(VaultSpec {
                src: None,
                dst: PathBuf::from("other/path"),
            })
        )
    }

    #[test]
    fn it_is_created_from_relative_source_and_relative_destination() {
        assert_eq!(
            VaultSpec::try_from("relative/path:other/path"),
            Ok(VaultSpec {
                src: Some(PathBuf::from("relative/path")),
                dst: PathBuf::from("other/path"),
            })
        )
    }

    #[test]
    fn it_is_created_from_relative_source_fills_destination_with_source_when_using_a_single_colon() {
        assert_eq!(
            VaultSpec::try_from("relative/path:"),
            Ok(VaultSpec {
                src: Some(PathBuf::from("relative/path")),
                dst: PathBuf::from("relative/path"),
            })
        )
    }

    #[test]
    fn it_is_created_from_relative_source_fills_destination_with_source() {
        assert_eq!(
            VaultSpec::try_from("relative/path"),
            Ok(VaultSpec {
                src: Some(PathBuf::from("relative/path")),
                dst: PathBuf::from("relative/path"),
            })
        )
    }

    #[test]
    fn it_does_not_allow_an_absolute_destination_even_if_it_is_inferred() {
        let invalid = "/absolute/path";
        assert_eq!(
            VaultSpec::try_from(invalid),
            Err(VaultSpecError(format!(
                "'{}' must not have an absolute destination. Try specifying the destination explicitly.",
                invalid
            )))
        )
    }

    #[test]
    fn it_do_allow_relative_parent_directories_if_destination_is_specified() {
        assert_eq!(
            VaultSpec::try_from("../relative:destination"),
            Ok(VaultSpec {
                src: Some(PathBuf::from("../relative")),
                dst: PathBuf::from("destination"),
            })
        )
    }
    #[test]
    fn it_does_allow_relative_parent_directories_in_destinations_if_that_is_what_the_user_wants() {
        assert_eq!(
            VaultSpec::try_from("../relative:../other"),
            Ok(VaultSpec {
                src: Some(PathBuf::from("../relative")),
                dst: PathBuf::from("../other"),
            })
        )
    }

    #[test]
    fn it_does_not_allow_relative_parent_directories() {
        assert_eq!(
            VaultSpec::try_from("../relative"),
            Err(VaultSpecError(format!(
                "Relative parent directories in source '../relative' need the destination set explicitly.",
            )))
        )
    }

    #[test]
    fn it_handles_an_empty_string_too() {
        assert_eq!(
            VaultSpec::try_from(""),
            Err(VaultSpecError(format!("An empty spec is invalid.",)))
        )
    }

    #[test]
    fn it_does_allow_an_absolute_destination_if_it_is_specified() {
        let invalid = "relative/path:/absolute/path";
        assert_eq!(
            VaultSpec::try_from(invalid),
            Ok(VaultSpec {
                src: Some(PathBuf::from("relative/path")),
                dst: PathBuf::from("/absolute/path"),
            })
        )
    }

    #[test]
    fn it_displays_itself_properly() {
        for &(ref input, ref expected) in
            [
                (":path", ":path"),
                ("src:dst", "src:dst"),
                ("src", "src:src"),
                ("src:", "src:src"),
            ].iter()
        {
            let s = VaultSpec::try_from(*input).unwrap();
            assert_eq!(&format!("{}", s), expected)
        }
    }
}
