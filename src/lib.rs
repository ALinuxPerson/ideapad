//! Utilities for IdeaPad laptops.

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[macro_use]
pub mod macros {
    #[macro_export]
    macro_rules! borrowed_cow_array {
        () => {
            &[]
        };
        ($($item:expr),+ $(,)?) => {
            &[$(::std::borrow::Cow::Borrowed($item)),+]
        }
    }

    #[macro_export]
    macro_rules! borrowed_cow_vec {
        () => {
            vec![]
        };
        ($($item:expr),+ $(,)?) => {
            vec![$(::std::borrow::Cow::Borrowed($item)),+]
        }
    }
}

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
    let _ = OldProfile::auto_detect()?;

    Ok(())
}

pub fn initialize_with_profile(profile: OldProfile) {
    let _ = OldProfile::initialize_with_profile(profile);
}

pub fn initialize_with_search_path(search_path: impl Iterator<Item =OldProfile>) -> profile::Result<()> {
    let _ = OldProfile::initialize_with_search_path(search_path)?;

    Ok(())
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Handler {
    Ignore,
    Error,
    Switch,
}
