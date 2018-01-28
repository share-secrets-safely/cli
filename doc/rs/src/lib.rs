extern crate mdbook;
use mdbook::{BookItem, MDBook};
use mdbook::book::Book;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::errors::Result as MdBookResult;
use std::path::PathBuf;

struct RunShellScript;

impl Preprocessor for RunShellScript {
    fn name(&self) -> &str {
        "run_shell_scripts"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: &mut Book) -> MdBookResult<()> {
        book.for_each_mut(|item: &mut BookItem| {
            println!("{:?}", item)
        });
        Ok(())
    }
}

pub fn build(dir: PathBuf) -> MdBookResult<()> {
    let mut md = MDBook::load(dir).expect("valid book");
    md.with_preprecessor(RunShellScript);
    md.build()
}
