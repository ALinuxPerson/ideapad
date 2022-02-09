//! Control rapid charge.
//!
//! Rapid charge charges your battery faster somehow.

use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::battery::enable::{Begin, EnableBuilder};
use crate::battery::{BatteryController, BatteryEnableGuard};
use crate::context::Context;
use try_drop::prelude::*;
use crate::Handler;
use thiserror::Error;
use try_drop::{DropAdapter, GlobalFallbackTryDropStrategyHandler, GlobalTryDropStrategyHandler};
use crate::battery_conservation::BatteryConservationDisableGuardInner;

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
pub type EnableRapidChargeBuilder<'rc, 'ctx, D, DD, S> =
    EnableBuilder<'rc, 'ctx, S, RapidChargeController<'ctx, D, DD>, D, DD>;

/// Inner value of [`RapidChargeEnableGuard`].
pub struct RapidChargeEnableGuardInner<'rc, 'ctx, D, DD>
where
    'ctx: 'rc,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    /// Reference to the rapid charge controller.
    pub controller: &'rc mut RapidChargeController<'ctx, D, DD>,
}

/// Guarantees that rapid charge is enabled for the scope
/// (excluding external access to `/proc/acpi/call`).
pub struct RapidChargeEnableGuard<'rc, 'ctx, D = GlobalTryDropStrategyHandler, DD = GlobalFallbackTryDropStrategyHandler>(DropAdapter<RapidChargeEnableGuardInner<'rc, 'ctx, D, DD>>)
where
    'ctx: 'rc,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy;

impl<'rc, 'ctx, D, DD> PureTryDrop for RapidChargeEnableGuardInner<'rc, 'ctx, D, DD>
    where
        'ctx: 'rc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    type Error = acpi_call::Error;
    type FallbackTryDropStrategy = DD;
    type TryDropStrategy = D;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &self.controller.context.fallback_try_drop_strategy
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &self.controller.context.fallible_try_drop_strategy
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.controller.disable()
    }
}

impl<'rc, 'ctx, D, DD> BatteryEnableGuard<'rc, 'ctx, RapidChargeController<'ctx, D, DD>>
    for RapidChargeEnableGuard<'rc, 'ctx, D, DD>
    where
        'ctx: 'rc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    type Inner = BatteryConservationDisableGuardInner<'rc, 'ctx, D, DD>;

    fn new(
        controller: &'rc mut RapidChargeController<'ctx, D, DD>,
        handler: Handler,
    ) -> Result<Self> {
        controller.enable().handler(handler).now()?;
        Ok(Self(DropAdapter(RapidChargeEnableGuardInner { controller })))
    }
}

/// Controller for rapid charge.
#[derive(Copy, Clone)]
pub struct RapidChargeController<'ctx, D = GlobalTryDropStrategyHandler, DD = GlobalFallbackTryDropStrategyHandler>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    /// Reference to the context.
    pub context: &'ctx Context<D, DD>,
}

impl<'ctx, D, DD> RapidChargeController<'ctx, D, DD>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    /// Create a new controller.
    pub fn new(context: &'ctx Context<D, DD>) -> Self {
        Self { context }
    }

    /// Builder for enabling rapid charge.
    pub fn enable<'rc>(&'rc mut self) -> EnableRapidChargeBuilder<'rc, 'ctx, D, DD, Begin> {
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

impl<'this, 'ctx, D, DD> BatteryController<'this, 'ctx> for RapidChargeController<'ctx, D, DD>
where
    'ctx: 'this,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    type EnableGuard = RapidChargeEnableGuard<'this, 'ctx, D, DD>;
    type Error = Error;

    fn enable_ignore(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.context.profile.battery.set_command.to_string(),
            [self.context.profile.battery.rapid_charge.parameters.enable],
        )?;

        Ok(())
    }

    fn enable_error(&mut self) -> std::result::Result<(), Self::Error> {
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
pub fn enable<D, DD>(context: &Context<D, DD>) -> Result<()>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    context.controllers().rapid_charge().enable().switch().now()
}

/// Disable rapid charge.
pub fn disable<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<()>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    context.controllers().rapid_charge().disable()
}

/// Get the rapid charge status.
pub fn get<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<bool>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    context.controllers().rapid_charge().get()
}

/// Check if rapid charge is enabled.
pub fn enabled<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<bool>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    context.controllers().rapid_charge().enabled()
}

/// Check if rapid charge is disabled.
pub fn disabled<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<bool>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
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
