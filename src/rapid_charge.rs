//! Control rapid charge.
//!
//! Rapid charge charges your battery faster somehow.

use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::Handler;
use thiserror::Error;
use crate::context::Context;

/// Handy wrapper for [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Bad things that could happen when controlling rapid charge.
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred when dealing with `acpi_call`.
    #[error("{error}")]
    AcpiCall {
        /// The underlying error itself.
        #[from]
        error: acpi_call::Error,
    },

    /// Occurs when you try to enable rapid charge when you have battery conservation already
    /// enabled.
    ///
    /// Only appears when you use [`RapidChargeController::enable_error`] or
    /// [`RapidChargeController::enable_with_handler`] with [`Handler::Error`].
    #[error("battery conservation is enabled, disable it before enabling rapid charge")]
    BatteryConservationEnabled,
}

/// Controller for rapid charge.
#[derive(Copy, Clone)]
pub struct RapidChargeController<'ctx> {
    pub context: &'ctx Context,
}

impl<'ctx> RapidChargeController<'ctx> {
    /// Create a new controller.
    pub const fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Enable rapid charge with the specified [`Handler`].
    ///
    /// For more information on what do the [`Handler`]s mean, see the [`Handler`] documentation.
    pub fn enable_with_handler(&mut self, handler: Handler) -> Result<()> {
        match handler {
            Handler::Ignore => self.enable_ignore().map_err(Into::into),
            Handler::Error => self.enable_error(),
            Handler::Switch => self.enable_switch().map_err(Into::into),
        }
    }

    /// Enable rapid charge, ignoring if battery conservation is already enabled.
    ///
    /// # Note
    /// Using this could drain your battery unnecessarily if battery conservation is enabled. Be
    /// careful!
    pub fn enable_ignore(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.context.profile.battery.set_command.to_string(),
            [self.context.profile.battery.rapid_charge.parameters.enable],
        )?;

        Ok(())
    }

    /// Enable battery conservation, returning an [`Error::BatteryConservationEnabled`] if rapid
    /// charge is already enabled.
    pub fn enable_error(&mut self) -> Result<()> {
        if self.context.controllers().battery_conservation().enabled()? {
            Err(Error::BatteryConservationEnabled)
        } else {
            self.enable_ignore().map_err(Into::into)
        }
    }

    /// Enable rapid charge, switching off battery conservation if it is enabled.
    pub fn enable_switch(&mut self) -> acpi_call::Result<()> {
        let mut battery_conservation = self.context.controllers().battery_conservation();

        if battery_conservation.enabled()? {
            battery_conservation.disable()?
        }

        self.enable_ignore()
    }

    /// Disable rapid charge.
    pub fn disable(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.context.profile.battery.set_command.to_string(),
            [self.context.profile.battery.rapid_charge.parameters.disable],
        )?;

        Ok(())
    }

    /// Get the rapid charge status.
    pub fn get(&self) -> acpi_call::Result<bool> {
        let output = acpi_call_expect_valid(
            self.context.profile.battery.rapid_charge.get_command.to_string(),
            [],
        )?;

        Ok(output != 0)
    }

    /// Check if rapid charge is enabled.
    pub fn enabled(&self) -> acpi_call::Result<bool> {
        self.get()
    }

    /// Check if rapid charge is disabled.
    pub fn disabled(&self) -> acpi_call::Result<bool> {
        self.get().map(|enabled| !enabled)
    }
}

// /// Uses the global profile. See [`RapidChargeController::enable_with_handler`] for documentation.
// pub fn enable_with_handler(handler: Handler) -> Result<()> {
//     Profile::get().rapid_charge().enable_with_handler(handler)
// }
//
// /// Uses the global profile. See [`RapidChargeController::enable_switch`] for documentation.
// pub fn enable_switch() -> acpi_call::Result<()> {
//     Profile::get().rapid_charge().enable_switch()
// }
//
// /// Uses the global profile. See [`RapidChargeController::enable_ignore`] for documentation.
// pub fn enable_ignore() -> acpi_call::Result<()> {
//     Profile::get().rapid_charge().enable_ignore()
// }
//
// /// Uses the global profile. See [`RapidChargeController::enable_error`] for documentation.
// pub fn enable_error() -> Result<()> {
//     Profile::get().rapid_charge().enable_error()
// }
//
// /// Uses the global profile. See [`RapidChargeController::disable`] for documentation.
// pub fn disable() -> acpi_call::Result<()> {
//     Profile::get().rapid_charge().disable()
// }
//
// /// Uses the global profile. See [`RapidChargeController::get`] for documentation.
// pub fn get() -> acpi_call::Result<bool> {
//     Profile::get().rapid_charge().get()
// }
//
// /// Uses the global profile. See [`RapidChargeController::enabled`] for documentation.
// pub fn enabled() -> acpi_call::Result<bool> {
//     Profile::get().rapid_charge().enabled()
// }
//
// /// Uses the global profile. See [`RapidChargeController::disabled`] for documentation.
// pub fn disabled() -> acpi_call::Result<bool> {
//     Profile::get().rapid_charge().disabled()
// }

#[cfg(test)]
mod tests {
    #[cfg(test)]
    fn test_enable_with_handler() {
        todo!()
    }

    #[cfg(test)]
    fn test_enable_ignore() {
        todo!()
    }

    #[cfg(test)]
    fn test_enable_error() {
        todo!()
    }

    #[cfg(test)]
    fn test_enable_switch() {
        todo!()
    }

    #[cfg(test)]
    fn test_disable() {
        todo!()
    }

    #[cfg(test)]
    fn test_get() {
        todo!()
    }

    #[cfg(test)]
    fn test_enabled() {
        todo!()
    }

    #[cfg(test)]
    fn test_disabled() {
        todo!()
    }
}
