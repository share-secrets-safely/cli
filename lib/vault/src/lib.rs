#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate gpgme;
extern crate s3_types;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

mod error;
mod util;
mod types;
mod dispatch;
mod init;
mod resource;

pub use init::init;
pub use dispatch::do_it;
