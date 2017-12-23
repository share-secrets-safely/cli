use std::io;
use std::path::Path;
use std::fs::{self, OpenOptions};

pub fn write_at(path: &Path) -> io::Result<fs::File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
}
