#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate glob;
extern crate gpgme;
extern crate itertools;
extern crate s3_types;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate yaml_rust;

mod error;
mod util;
mod vault;
mod dispatch;
mod init;
mod resource;

pub use vault::Vault;
pub use dispatch::do_it;
