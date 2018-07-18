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

mod base;
pub mod error;
mod init;
mod partitions;
mod recipients;
mod resource;
mod spec;
mod util;

pub use base::{TrustModel, Vault, VaultExt};
pub use spec::*;
pub use util::print_causes;
