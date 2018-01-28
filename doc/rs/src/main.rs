extern crate doc;
use std::env;
use std::path::PathBuf;

fn main() {
    let root_dir = env::args()
        .skip(1)
        .next()
        .expect("book root directory as first argument");

    doc::build(PathBuf::from(root_dir)).expect("valid book");
}
