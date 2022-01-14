#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate serial_test;

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

/// Initializes this crate with an auto detected profile. Note that if you don't intend on using
/// the global profile, you don't need to call this function.
pub fn initialize() -> profile::Result<()> {
    let _ = Profile::auto_detect()?;

    Ok(())
}

/// Initialize the global profile with the specified profile.
pub fn initialize_with_profile(profile: Profile) {
    let _ = Profile::initialize_with_profile(profile);
}

/// Initialize the global profile with the specified search path.
pub fn initialize_with_search_path(search_path: impl Iterator<Item = Profile>) -> profile::Result<()> {
    let _ = Profile::initialize_with_search_path(search_path)?;

    Ok(())
}

/// Handlers which determine what to do when battery conservation and rapid charge modes conflict.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Handler {
    /// Ignore the conflict and continue with the current mode.
    Ignore,

    /// Return an error.
    Error,

    /// Switch the conflicting mode to disabled then try again.
    Switch,
}
