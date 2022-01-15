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
pub mod battery;
pub mod context;
pub mod fallible_drop_strategy;
pub mod prelude;
pub mod profile;
pub mod rapid_charge;
pub mod system_performance;

#[cfg(test)]
mod battery_conservation_rapid_charge_shared_tests;

use parking_lot::RwLockReadGuard;
pub use prelude::*;

#[cfg(not(target_os = "linux"))]
compile_error!(
    "this crate only works on linux systems due to its dependency on the `acpi_call` kernel module"
);

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
