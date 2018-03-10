extern crate conv;
extern crate sheesy_vault;

use sheesy_vault::{SpecSourceType, VaultSpec, VaultSpecError};

use conv::TryFrom;
use std::path::PathBuf;

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
        Err(VaultSpecError(format!(
            "'{}' does not contain a destination.",
            invalid
        ),))
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
            src: SpecSourceType::Stdin,
            dst: PathBuf::from("other/path"),
        })
    )
}

#[test]
fn it_is_created_from_relative_source_and_relative_destination() {
    assert_eq!(
        VaultSpec::try_from("relative/path:other/path"),
        Ok(VaultSpec {
            src: SpecSourceType::Path(PathBuf::from("relative/path")),
            dst: PathBuf::from("other/path"),
        })
    )
}

#[test]
fn it_is_created_from_relative_source_fills_destination_with_source_when_using_a_single_colon() {
    assert_eq!(
        VaultSpec::try_from("relative/path:"),
        Ok(VaultSpec {
            src: SpecSourceType::Path(PathBuf::from("relative/path")),
            dst: PathBuf::from("relative/path"),
        })
    )
}

#[test]
fn it_is_created_from_relative_source_fills_destination_with_source() {
    assert_eq!(
        VaultSpec::try_from("relative/path"),
        Ok(VaultSpec {
            src: SpecSourceType::Path(PathBuf::from("relative/path")),
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
            src: SpecSourceType::Path(PathBuf::from("../relative")),
            dst: PathBuf::from("destination"),
        })
    )
}
#[test]
fn it_does_allow_relative_parent_directories_in_destinations_if_that_is_what_the_user_wants() {
    assert_eq!(
        VaultSpec::try_from("../relative:../other"),
        Ok(VaultSpec {
            src: SpecSourceType::Path(PathBuf::from("../relative")),
            dst: PathBuf::from("../other"),
        })
    )
}

#[test]
fn it_does_not_allow_relative_parent_directories() {
    assert_eq!(
        VaultSpec::try_from("../relative"),
        Err(VaultSpecError(
            "Relative parent directories in source '../relative' need the destination set explicitly.".to_owned(),
        ))
    )
}

#[test]
fn it_handles_an_empty_string_too() {
    assert_eq!(
        VaultSpec::try_from(""),
        Err(VaultSpecError("An empty spec is invalid.".to_owned(),))
    )
}

#[test]
fn it_does_allow_an_absolute_destination_if_it_is_specified() {
    let invalid = "relative/path:/absolute/path";
    assert_eq!(
        VaultSpec::try_from(invalid),
        Ok(VaultSpec {
            src: SpecSourceType::Path(PathBuf::from("relative/path")),
            dst: PathBuf::from("/absolute/path"),
        })
    )
}

#[test]
fn it_displays_itself_properly() {
    for &(input, expected) in &[
        (":path", ":path"),
        ("src:dst", "src:dst"),
        ("src", "src:src"),
        ("src:", "src:src"),
    ] {
        let s = VaultSpec::try_from(input).unwrap();
        assert_eq!(&format!("{}", s), expected)
    }
}
