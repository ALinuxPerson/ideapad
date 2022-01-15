//! Most commonly used types.

pub use crate::{
    context::Context,
    fallible_drop_strategy::{
        DynFallibleDropStrategy,
        FallibleDropStrategy,
        ThreadSafeWrite,
        ThreadSafe,
        FallibleDropStrategies,
    },
    profile::{
        Error as ProfileError,
        Profile,
        Result as ProfileResult,
    },
};

#[cfg(feature = "battery_conservation")]
pub use crate::battery_conservation::{
    BatteryConservationController,
    Error as BatteryConservationModeError,
    Result as BatteryConservationModeResult,
};

#[cfg(feature = "rapid_charge")]
pub use crate::rapid_charge::{
    Error as RapidChargeError,
    RapidChargeController,
    Result as RapidChargeResult
};

#[cfg(feature = "system_performance")]
pub use crate::system_performance::{
    Error as SystemPerformanceModeError, Result as SystemPerformanceModeResult,
    SystemPerformanceController, SystemPerformanceMode,
};

#[cfg(any(feature = "battery_conservation", feature = "rapid_charge", feature = "system_performance"))]
pub use crate::acpi_call::{Error as AcpiCallError, Result as AcpiCallResult};

#[cfg(any(feature = "battery_conservation", feature = "rapid_charge"))]
pub use crate::Handler;