mod util;
#[cfg(feature = "vault")]
pub mod vault;
#[cfg(feature = "rest")]
pub mod completions;
#[cfg(feature = "rest")]
pub mod substitute;
#[cfg(feature = "rest")]
pub mod merge;
#[cfg(feature = "rest")]
pub mod extract;
