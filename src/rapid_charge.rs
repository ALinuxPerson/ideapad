//! Control rapid charge.
//!
//! Rapid charge charges your battery faster somehow.

use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::battery::enable::{Begin, EnableBuilder};
use crate::battery::{BatteryController, BatteryEnableGuard};
use crate::context::Context;
use crate::fallible_drop_strategy::{FallibleDropStrategies, FallibleDropStrategy};
use crate::Handler;
use thiserror::Error;

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

/// Builder for enabling rapid charge.
pub type EnableRapidChargeBuilder<'rc, 'ctx, S> =
    EnableBuilder<'rc, 'ctx, S, RapidChargeController<'ctx>>;

/// Guarantees that rapid charge is enabled for the scope
/// (excluding external access to `/proc/acpi/call`).
pub struct RapidChargeEnableGuard<'rc, 'ctx: 'rc> {
    /// Reference to the rapid charge controller.
    pub controller: &'rc mut RapidChargeController<'ctx>,
}

impl<'rc, 'ctx: 'rc> RapidChargeEnableGuard<'rc, 'ctx> {
    fn fallible_drop_strategy(&self) -> &'ctx FallibleDropStrategies {
        self.controller.context.fallible_drop_strategy()
    }
}

impl<'rc, 'ctx: 'rc> Drop for RapidChargeEnableGuard<'rc, 'ctx> {
    fn drop(&mut self) {
        self.fallible_drop_strategy()
            .handle_error(self.controller.disable())
    }
}

impl<'rc, 'ctx: 'rc> BatteryEnableGuard<'rc, 'ctx, RapidChargeController<'ctx>>
    for RapidChargeEnableGuard<'rc, 'ctx>
{
    type Error = Error;

    fn new(
        controller: &'rc mut RapidChargeController<'ctx>,
        handler: Handler,
    ) -> std::result::Result<Self, Self::Error> {
        controller.enable().handler(handler).now()?;
        Ok(Self { controller })
    }
}

/// Controller for rapid charge.
#[derive(Copy, Clone)]
pub struct RapidChargeController<'ctx> {
    /// Reference to the context.
    pub context: &'ctx Context,
}

impl<'ctx> RapidChargeController<'ctx> {
    /// Create a new controller.
    pub const fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Builder for enabling rapid charge.
    pub fn enable<'rc>(&'rc mut self) -> EnableRapidChargeBuilder<'rc, 'ctx, Begin> {
        EnableRapidChargeBuilder::new(self)
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
            self.context
                .profile
                .battery
                .rapid_charge
                .get_command
                .to_string(),
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

impl<'this, 'ctx: 'this> BatteryController<'this, 'ctx> for RapidChargeController<'ctx> {
    type EnableGuard = RapidChargeEnableGuard<'this, 'ctx>;
    type EnableError = Error;

    fn enable_ignore(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.context.profile.battery.set_command.to_string(),
            [self.context.profile.battery.rapid_charge.parameters.enable],
        )?;

        Ok(())
    }

    fn enable_error(&mut self) -> std::result::Result<(), Self::EnableError> {
        if self
            .context
            .controllers()
            .battery_conservation()
            .enabled()?
        {
            Err(Error::BatteryConservationEnabled)
        } else {
            self.enable_ignore().map_err(Into::into)
        }
    }

    fn enable_switch(&mut self) -> acpi_call::Result<()> {
        let mut battery_conservation = self.context.controllers().battery_conservation();

        if battery_conservation.enabled()? {
            battery_conservation.disable()?
        }

        self.enable_ignore()
    }
}

/// Enable rapid charge, switching off battery conservation if it's enabled.
///
/// For more advanced usage, see [`RapidChargeController::enable`].
pub fn enable(context: &Context) -> Result<()> {
    context.controllers()
        .rapid_charge()
        .enable()
        .switch()
        .now()
}

/// Disable rapid charge.
pub fn disable(context: &Context) -> acpi_call::Result<()> {
    context.controllers().rapid_charge().disable()
}

/// Get the rapid charge status.
pub fn get(context: &Context) -> acpi_call::Result<bool> {
    context.controllers().rapid_charge().get()
}

/// Check if rapid charge is enabled.
pub fn enabled(context: &Context) -> acpi_call::Result<bool> {
    context.controllers().rapid_charge().enabled()
}

/// Check if rapid charge is disabled.
pub fn disabled(context: &Context) -> acpi_call::Result<bool> {
    context.controllers().rapid_charge().disabled()
}

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
