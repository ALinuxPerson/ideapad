//! Most commonly used types.

pub use crate::{
    acpi_call::{Error as AcpiCallError, Result as AcpiCallResult},
    battery_conservation::{
        BatteryConservationController, Error as BatteryConservationModeError,
        Result as BatteryConservationModeResult,
    },
    profile::{
        Profile,
        Error as ProfileError,
        Result as ProfileResult,
    },
    rapid_charge::{Error as RapidChargeError, RapidChargeController, Result as RapidChargeResult},
    system_performance::{
        Error as SystemPerformanceModeError, Result as SystemPerformanceModeResult,
        SystemPerformanceMode, SystemPerformanceController,
    },
    Handler,
};
