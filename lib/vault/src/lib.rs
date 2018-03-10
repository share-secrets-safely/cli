extern crate atty;
extern crate conv;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate glob;
extern crate gpgme;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate mktemp;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate yaml_rust;

pub mod error;
mod util;
mod base;
pub mod dispatch;
mod recipients;
mod spec;
mod init;
mod resource;
mod partitions;

pub use spec::*;
pub use base::Vault;
pub use util::print_causes;
