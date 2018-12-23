#![forbid(unsafe_code)]

#[macro_use]
extern crate failure;
extern crate serde_json as json;
extern crate serde_yaml as yaml;

#[cfg(feature = "substitute")]
extern crate atty;
#[cfg(feature = "substitute")]
extern crate base64;
#[cfg(feature = "process")]
extern crate glob;
#[cfg(feature = "substitute")]
extern crate handlebars;
#[cfg(feature = "substitute")]
extern crate liquid;
#[cfg(feature = "substitute")]
extern crate liquid_error;
#[cfg(feature = "process")]
extern crate serde;
#[cfg(feature = "process")]
extern crate treediff;
#[cfg(any(feature = "substitute", feature = "process"))]
extern crate yaml_rust;

#[cfg(feature = "process")]
pub mod process;
#[cfg(feature = "substitute")]
pub mod substitute;
