extern crate sheesy_substitute;

use sheesy_substitute::Spec;
use sheesy_substitute::StreamOrPath::*;
use std::path::PathBuf;

#[cfg(test)]
mod parse {
    use super::*;

    #[test]
    fn empty() {
        let actual = Spec::from("");
        assert_eq!(
            actual,
            Spec {
                src: Stream,
                dst: Stream,
            }
        );

        assert_eq!(format!("{}", actual), ":")
    }

    #[test]
    fn colon() {
        let actual = Spec::from(":");
        assert_eq!(
            actual,
            Spec {
                src: Stream,
                dst: Stream,
            }
        );

        assert_eq!(format!("{}", actual), ":")
    }

    #[test]
    fn stream_path() {
        let actual = Spec::from(":foo");
        assert_eq!(
            actual,
            Spec {
                src: Stream,
                dst: Path(PathBuf::from("foo")),
            }
        );

        assert_eq!(format!("{}", actual), ":foo")
    }

    #[test]
    fn path_stream() {
        let actual = Spec::from("foo:");
        assert_eq!(
            actual,
            Spec {
                src: Path(PathBuf::from("foo")),
                dst: Stream,
            }
        );

        assert_eq!(format!("{}", actual), "foo")
    }

    #[test]
    fn path_path() {
        let actual = Spec::from("foo:bar");
        assert_eq!(
            actual,
            Spec {
                src: Path(PathBuf::from("foo")),
                dst: Path(PathBuf::from("bar")),
            }
        );

        assert_eq!(format!("{}", actual), "foo:bar")
    }

    #[test]
    fn absolute_path_absolute_path() {
        let actual = Spec::from("/foo/sub:/bar/sub");
        assert_eq!(
            actual,
            Spec {
                src: Path(PathBuf::from("/foo/sub")),
                dst: Path(PathBuf::from("/bar/sub")),
            }
        );

        assert_eq!(format!("{}", actual), "/foo/sub:/bar/sub")
    }
}
