#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[macro_use]
pub mod macros;

pub mod acpi_call;
pub mod battery_conservation;
pub mod prelude;
pub mod profile;
pub mod rapid_charge;
pub mod system_performance;

pub use prelude::*;

#[cfg(not(target_os = "linux"))]
compile_error!("this crate only works on linux systems due to its dependency on the `acpi_call` kernel module");

pub fn initialize() -> profile::Result<()> {
    let _ = Profile::auto_detect()?;

    Ok(())
}

pub fn initialize_with_profile(profile: Profile) {
    let _ = Profile::initialize_with_profile(profile);
}

pub fn initialize_with_search_path(search_path: impl Iterator<Item =Profile>) -> profile::Result<()> {
    let _ = Profile::initialize_with_search_path(search_path)?;

    Ok(())
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Handler {
    Ignore,
    Error,
    Switch,
}
