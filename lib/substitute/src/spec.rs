use std::path::PathBuf;
use std::fmt::{self, Write};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StreamOrPath {
    Stream,
    Path(PathBuf),
}

impl<'a> From<&'a str> for StreamOrPath {
    fn from(s: &str) -> Self {
        use self::StreamOrPath::*;
        if s.is_empty() {
            Stream
        } else {
            Path(PathBuf::from(s))
        }
    }
}

impl fmt::Display for StreamOrPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::StreamOrPath::*;
        match *self {
            Stream => Ok(()),
            Path(ref p) => p.display().fmt(f),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Spec {
    pub src: StreamOrPath,
    pub dst: StreamOrPath,
}

impl Spec {
    pub fn sep() -> char {
        ':'
    }
}

impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::StreamOrPath::*;
        match (&self.src, &self.dst) {
            (&Stream, &Stream) => f.write_char(Spec::sep()),
            (&Path(ref p), &Stream) => p.display().fmt(f),
            (&Stream, &Path(ref p)) => f.write_char(Spec::sep()).and(p.display().fmt(f)),
            (&Path(ref p1), &Path(ref p2)) => p1.display()
                .fmt(f)
                .and(f.write_char(Spec::sep()))
                .and(p2.display().fmt(f)),
        }
    }
}

impl<'a> From<&'a str> for Spec {
    fn from(src: &'a str) -> Self {
        use self::StreamOrPath::*;
        let mut it = src.splitn(2, Spec::sep());
        match (it.next(), it.next()) {
            (None, Some(_)) | (None, None) => unreachable!(),
            (Some(p), None) => Spec {
                src: StreamOrPath::from(p),
                dst: Stream,
            },
            (Some(p1), Some(p2)) => Spec {
                src: StreamOrPath::from(p1),
                dst: StreamOrPath::from(p2),
            },
        }
    }
}
