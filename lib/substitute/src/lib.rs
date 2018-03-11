extern crate atty;
#[macro_use]
extern crate failure;
extern crate handlebars;
extern crate serde_json as json;
extern crate serde_yaml as yaml;

mod spec;
mod substitute;

pub use spec::*;
pub use substitute::*;
