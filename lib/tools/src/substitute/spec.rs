use failure::{Error, ResultExt};
use atty;

use std::path::PathBuf;
use std::fmt;
use std::ffi::OsStr;
use std::io::{self, stdin, stdout};
use std::fs::{File, OpenOptions};
use std::fs::create_dir_all;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StreamOrPath {
    Stream,
    Path(PathBuf),
}

impl StreamOrPath {
    pub fn is_stream(&self) -> bool {
        match *self {
            StreamOrPath::Stream => true,
            StreamOrPath::Path(_) => false,
        }
    }

    pub fn name(&self) -> &str {
        match *self {
            StreamOrPath::Stream => "stream",
            StreamOrPath::Path(ref p) => p.to_str().unwrap_or("<file-path-is-not-unicode>"),
        }
    }

    pub fn short_name(&self) -> &str {
        match *self {
            StreamOrPath::Stream => "stream",
            StreamOrPath::Path(ref p) => p.file_stem().map_or("<invalid-file-stem>", |s| {
                s.to_str().unwrap_or("<file-stem-is-not-unicode>")
            }),
        }
    }

    pub fn open_as_output(&self, append: bool) -> Result<Box<io::Write>, Error> {
        Ok(match *self {
            StreamOrPath::Stream => Box::new(stdout()),
            StreamOrPath::Path(ref p) => {
                if let Some(dir) = p.parent() {
                    create_dir_all(dir)
                        .context(format!("Could not create directory leading towards '{}'", p.display(),))?;
                }
                Box::new(OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(append)
                    .open(p)
                    .context(format!("Could not open '{}' for writing", p.display()))?)
            }
        })
    }

    pub fn open_as_input(&self) -> Result<Box<io::Read>, Error> {
        Ok(match *self {
            StreamOrPath::Stream => if atty::is(atty::Stream::Stdin) {
                bail!("Cannot read from standard input while a terminal is connected")
            } else {
                Box::new(stdin())
            },
            StreamOrPath::Path(ref p) => {
                Box::new(File::open(p).context(format!("Could not open '{}' for reading", p.display()))?)
            }
        })
    }
}

impl<'a> From<&'a OsStr> for StreamOrPath {
    fn from(p: &'a OsStr) -> Self {
        StreamOrPath::Path(PathBuf::from(p))
    }
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
        use self::fmt::Write;
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
