//! Control battery conservation mode.
//!
//! Battery conservation mode is a mode found in Ideapad laptops which limits the battery's maximum
//! level to 60%. However, if you charge your battery above 60% with battery conservation mode
//! disabled then enable it, the battery level will be capped at the level you enabled battery
//! conservation mode at. For example, if you charge your battery to 80% and then enable battery
//! conservation mode, the battery level will be capped at 80%.

use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::profile::Profile;
use crate::Handler;
use thiserror::Error;

/// Handy wrapper for [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Bad things that could happen when dealing with battery conservation mode.
#[derive(Debug, Error)]
pub enum Error {
    /// An error returned from `acpi_call`.
    #[error("{error}")]
    AcpiCall {
        /// The error itself.
        #[from]
        error: acpi_call::Error,
    },

    /// Occurs when you try to enable battery conservation when you have rapid charge already
    /// enabled.
    ///
    /// Only appears when you use [`BatteryConservationController::enable_error`] or
    /// [`BatteryConservationController::enable_with_handler`] with [`Handler::Error`].
    #[error("rapid charge is enabled, disable it first before enabling battery conservation mode")]
    RapidChargeEnabled,
}

/// Controller for battery conservation mode.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BatteryConservationController<'p> {
    /// Reference to the profile.
    pub profile: &'p Profile,
}

impl<'p> BatteryConservationController<'p> {
    /// Create a new battery conservation controller.
    pub const fn new(profile: &'p Profile) -> Self {
        Self { profile }
    }

    /// Enable battery conservation with the specified [`Handler`].
    ///
    /// For more information on what do the [`Handler`]s mean, see the [`Handler`] documentation.
    pub fn enable_with_handler(&self, handler: Handler) -> Result<()> {
        match handler {
            Handler::Ignore => self.enable_ignore().map_err(Into::into),
            Handler::Error => self.enable_error(),
            Handler::Switch => self.enable_switch().map_err(Into::into),
        }
    }

    /// Enable battery conservation, ignoring if rapid charge is already enabled.
    ///
    /// # Note
    /// Using this could drain your battery unnecessarily if rapid charge is enabled. Be careful!
    pub fn enable_ignore(&self) -> acpi_call::Result<()> {
        acpi_call(
            self.profile.battery.set_command.to_string(),
            [self.profile.battery.conservation.parameters.enable],
        )?;

        Ok(())
    }

    /// Enable battery conservation, returning an [`Error::RapidChargeEnabled`] if rapid charge is
    /// already enabled.
    pub fn enable_error(&self) -> Result<()> {
        if self.profile.rapid_charge().enabled()? {
            Err(Error::RapidChargeEnabled)
        } else {
            self.enable_ignore().map_err(Into::into)
        }
    }

    /// Enable battery conservation, switching off rapid charge if it is enabled.
    pub fn enable_switch(&self) -> acpi_call::Result<()> {
        let rapid_charge = self.profile.rapid_charge();

        if rapid_charge.enabled()? {
            rapid_charge.disable()?;
        }

        self.enable_ignore()
    }

    /// Disable battery conservation.
    pub fn disable(&self) -> acpi_call::Result<()> {
        acpi_call(
            self.profile.battery.set_command.to_string(),
            [self.profile.battery.conservation.parameters.disable],
        )?;

        Ok(())
    }

    /// Get the battery conservation status.
    pub fn get(&self) -> acpi_call::Result<bool> {
        let output =
            acpi_call_expect_valid(
                    self.profile.battery.conservation.get_command.to_string(),
                []
            )?;

        Ok(output != 0)
    }

    /// Check if battery conservation is enabled.
    pub fn enabled(&self) -> acpi_call::Result<bool> {
        self.get()
    }

    /// Check if battery conservation is disabled.
    pub fn disabled(&self) -> acpi_call::Result<bool> {
        self.get().map(|enabled| !enabled)
    }
}

/// Uses the global profile. See [`BatteryConservationController::enable_with_handler`] for
/// documentation.
pub fn enable_with_handler(handler: Handler) -> Result<()> {
    Profile::get()
        .battery_conservation()
        .enable_with_handler(handler)
}

/// Uses the global profile. See [`BatteryConservationController::enable_ignore`] for documentation.
pub fn enable_ignore() -> acpi_call::Result<()> {
    Profile::get()
        .battery_conservation()
        .enable_ignore()
}

/// Uses the global profile. See [`BatteryConservationController::enable_error`] for documentation.
pub fn enable_error() -> Result<()> {
    Profile::get().battery_conservation().enable_error()
}

/// Uses the global profile. See [`BatteryConservationController::enable_switch`] for documentation.
pub fn enable_switch() -> acpi_call::Result<()> {
    Profile::get().battery_conservation().enable_switch()
}

/// Uses the global profile. See [`BatteryConservationController::disable`] for documentation.
pub fn disable() -> acpi_call::Result<()> {
    Profile::get().battery_conservation().disable()
}

/// Uses the global profile. See [`BatteryConservationController::get`] for documentation.
pub fn get() -> acpi_call::Result<bool> {
    Profile::get().battery_conservation().get()
}

/// Uses the global profile. See [`BatteryConservationController::enabled`] for documentation.
pub fn enabled() -> acpi_call::Result<bool> {
    Profile::get().battery_conservation().enabled()
}

/// Uses the global profile. See [`BatteryConservationController::disabled`] for documentation.
pub fn disabled() -> acpi_call::Result<bool> {
    Profile::get().battery_conservation().disabled()
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    fn test_enable_with_handler() { todo!() }

    #[cfg(test)]
    fn test_enable_ignore() { todo!() }

    #[cfg(test)]
    fn test_enable_error() { todo!() }

    #[cfg(test)]
    fn test_enable_switch() { todo!() }

    #[cfg(test)]
    fn test_disable() { todo!() }

    #[cfg(test)]
    fn test_get() { todo!() }

    #[cfg(test)]
    fn test_enabled() { todo!() }

    #[cfg(test)]
    fn test_disabled() { todo!() }
}
