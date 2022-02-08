//! Control battery conservation mode.
//!
//! Battery conservation mode is a mode found in Ideapad laptops which limits the battery's maximum
//! level to 60%. However, if you charge your battery above 60% with battery conservation mode
//! disabled then enable it, the battery level will be capped at the level you enabled battery
//! conservation mode at. For example, if you charge your battery to 80% and then enable battery
//! conservation mode, the battery level will be capped at 80%.
use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::battery::enable::EnableBuilder;
use crate::battery::{BatteryController, BatteryEnableGuard};
use crate::context::Context;
// use crate::fallible_drop_strategy::{FallibleDropStrategies, FallibleDropStrategy};
use crate::{battery_conservation, Handler};
use thiserror::Error;
use try_drop::{DropAdapter, GlobalFallbackTryDropStrategyHandler, GlobalTryDropStrategyHandler};
use try_drop::prelude::*;

/// Handy wrapper for [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Builder for enabling battery conservation.
pub type EnableBatteryConservationBuilder<'ctrl, 'ctx, D, DD, S> =
    EnableBuilder<'ctrl, 'ctx, S, BatteryConservationController<'ctx, D, DD>, D, DD>;

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

/// Inner value for [`BatteryConservationEnableGuard`].
pub struct BatteryConservationEnableGuardInner<'bc, 'ctx, D, DD>
    where
        'ctx: 'bc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    controller: &'bc mut BatteryConservationController<'ctx, D, DD>,
}

impl<'bc, 'ctx, D, DD> PureTryDrop for BatteryConservationEnableGuardInner<'bc, 'ctx, D, DD>
    where
        'ctx: 'bc,
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

/// "Guarantees" that the battery conservation mode is enabled for the scope.
#[must_use]
pub struct BatteryConservationEnableGuard<'bc, 'ctx, D, DD>(DropAdapter<BatteryConservationEnableGuardInner<'bc, 'ctx, D, DD>>)
    where
        'ctx: 'bc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy;

pub struct BatteryConservationDisableGuardInner<'bc, 'ctx, D, DD>
    where
        'ctx: 'bc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    controller: &'bc mut BatteryConservationController<'ctx, D, DD>,
    handler: Handler,
}

/// "Guarantees" that the battery conservation mode is disabled for the scope.
#[must_use]
pub struct BatteryConservationDisableGuard<'bc, 'ctx, D, DD>(DropAdapter<BatteryConservationDisableGuardInner<'bc, 'ctx, D, DD>>)
    where
        'ctx: 'bc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy;

impl<'bc, 'ctx, D, DD> BatteryConservationDisableGuard<'bc, 'ctx, D, DD>
    where
        'ctx: 'bc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    /// Disable battery conservation mode for the scope.
    pub fn new(
        controller: &'bc mut BatteryConservationController<'ctx, D, DD>,
        handler: Handler,
    ) -> acpi_call::Result<Self> {
        controller.disable()?;

        Ok(Self(DropAdapter(BatteryConservationDisableGuardInner {
            controller,
            handler,
        })))
    }
}

impl<'bc, 'ctx, D, DD> BatteryEnableGuard<'bc, 'ctx, BatteryConservationController<'ctx, D, DD>>
    for BatteryConservationEnableGuard<'bc, 'ctx, D, DD>
where
    'ctx: 'bc,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    type Inner = BatteryConservationEnableGuardInner<'bc, 'ctx, D, DD>;

    fn new(
        controller: &'bc mut BatteryConservationController<'ctx, D, DD>,
        handler: Handler,
    ) -> Result<Self> {
        controller.enable().handler(handler).now()?;

        Ok(Self(DropAdapter(BatteryConservationEnableGuardInner { controller })))
    }
}

impl<'bc, 'ctx, D, DD> PureTryDrop for BatteryConservationDisableGuardInner<'bc, 'ctx, D, DD>
    where
        'ctx: 'bc,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    type Error = battery_conservation::Error;
    type FallbackTryDropStrategy = DD;
    type TryDropStrategy = D;

    fn fallback_try_drop_strategy(&self) -> &Self::FallbackTryDropStrategy {
        &self.controller.context.fallback_try_drop_strategy
    }

    fn try_drop_strategy(&self) -> &Self::TryDropStrategy {
        &self.controller.context.fallible_try_drop_strategy
    }

    unsafe fn try_drop(&mut self) -> Result<(), Self::Error> {
        self.controller.enable().handler(self.handler).now()
    }
}

/// Controller for battery conservation mode.
#[derive(Copy, Clone)]
pub struct BatteryConservationController<'ctx, D = GlobalTryDropStrategyHandler, DD = GlobalFallbackTryDropStrategyHandler>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    /// A reference to the context.
    pub context: &'ctx Context<D, DD>,
}

impl<'ctx, D, DD> BatteryConservationController<'ctx, D, DD>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    /// Create a new battery conservation controller.
    pub fn new(context: &'ctx Context<D, DD>) -> Self {
        Self { context }
    }

    /// Builder for enabling battery conservation.
    pub fn enable<'bc>(
        &'bc mut self,
    ) -> EnableBatteryConservationBuilder<'bc, 'ctx, D, DD, crate::battery::enable::Begin> {
        EnableBatteryConservationBuilder::new(self)
    }

    /// Disable battery conservation.
    pub fn disable(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.context.profile.battery.set_command.to_string(),
            [self.context.profile.battery.conservation.parameters.disable],
        )?;

        Ok(())
    }

    /// Get the battery conservation status.
    pub fn get(&self) -> acpi_call::Result<bool> {
        let output = acpi_call_expect_valid(
            self.context
                .profile
                .battery
                .conservation
                .get_command
                .to_string(),
            [],
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

    /// Ensures that the battery conservation mode is disabled for this scope.
    pub fn disable_guard<'bc>(
        &'bc mut self,
        handler: Handler,
    ) -> acpi_call::Result<BatteryConservationDisableGuard<'bc, 'ctx, D, DD>> {
        BatteryConservationDisableGuard::new(self, handler)
    }
}

impl<'this, 'ctx, D, DD> BatteryController<'this, 'ctx> for BatteryConservationController<'ctx, D, DD>
where
    'ctx: 'this,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    type EnableGuard = BatteryConservationEnableGuard<'this, 'ctx, D, DD>;
    type Error = Error;

    fn enable_ignore(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.context.profile.battery.set_command.to_string(),
            [self.context.profile.battery.conservation.parameters.enable],
        )?;

        Ok(())
    }

    fn enable_error(&mut self) -> Result<(), Self::Error> {
        if self.context.controllers().rapid_charge().enabled()? {
            Err(Error::RapidChargeEnabled)
        } else {
            self.enable_ignore().map_err(Into::into)
        }
    }

    fn enable_switch(&mut self) -> acpi_call::Result<()> {
        let mut rapid_charge = self.context.controllers().rapid_charge();

        if rapid_charge.enabled()? {
            rapid_charge.disable()?;
        }

        self.enable_ignore()
    }
}

/// Enable battery conservation with the switch handler. If you want more advanced options, see
/// [`BatteryConservationController::enable`].
pub fn enable<D, DD>(context: &Context<D, DD>) -> Result<()>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    context
        .controllers()
        .battery_conservation()
        .enable()
        .switch()
        .now()
}

/// Disable battery conservation.
pub fn disable<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<()>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    context.controllers().battery_conservation().disable()
}

/// Get the battery conservation status.
pub fn get<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<bool>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    context.controllers().battery_conservation().get()
}

/// Check if battery conservation is enabled.
pub fn enabled<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<bool>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    context.controllers().battery_conservation().enabled()
}

/// Check if battery conservation is disabled.
pub fn disabled<D, DD>(context: &Context<D, DD>) -> acpi_call::Result<bool>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    context.controllers().battery_conservation().disabled()
}

#[cfg(test)]
mod tests {
    use crate::{battery_conservation, rapid_charge, Context, Handler};
    use once_cell::sync::Lazy;

    static CONTEXT: Lazy<Context> = Lazy::new(|| crate::context().expect("failed to get context"));

    fn context() -> &'static Context {
        &CONTEXT
    }

    #[test]
    #[serial]
    fn test_enable_with_handler() {
        let controllers = context().controllers();
        let mut battery_conservation = controllers.battery_conservation();
        let mut rapid_charge = controllers.rapid_charge();

        // set up our scenario here
        battery_conservation
            .enable()
            .handler(Handler::Ignore)
            .now()
            .expect("failed to enable battery conservation");

        // let's test first with ignorance
        rapid_charge
            .enable()
            .handler(Handler::Ignore)
            .now()
            .expect("rapid charge enable failed");

        assert!(
            rapid_charge
                .enabled()
                .expect("failed to get rapid charge status"),
            "expected rapid charge to be enabled with the ignore handler",
        );

        // TIL ideapad laptops already have a built in mechanism to switch off rapid charging when
        // trying to enable battery conservation, albeit this is easily bypassed by just switching
        // on battery conservation again afterwards, sooo we still need the switch handler
        assert!(
            battery_conservation
                .disabled()
                .expect("failed to get battery conservation status"),
            "expected battery conservation to be disabled with the ignore handler",
        );

        // now let's test with an error handler
        battery_conservation
            .enable()
            .handler(Handler::Ignore)
            .now()
            .expect("failed to enable battery conservation");

        let error = rapid_charge
            .enable()
            .handler(Handler::Error)
            .now()
            .expect_err("rapid charge enable succeeded");
        assert!(matches!(
            error,
            rapid_charge::Error::BatteryConservationEnabled
        ));
        assert!(battery_conservation
            .enabled()
            .expect("failed to get battery conservation status"));

        // now let's test with a switch handler
        rapid_charge
            .enable()
            .handler(Handler::Switch)
            .now()
            .expect("rapid charge enable failed");
        assert!(rapid_charge
            .enabled()
            .expect("failed to get rapid charge status"));
        assert!(battery_conservation
            .disabled()
            .expect("failed to get battery conservation status"));
    }

    #[test]
    #[serial]
    fn test_enable_ignore() {
        let controllers = context().controllers();
        let mut battery_conservation = controllers.battery_conservation();
        let mut rapid_charge = controllers.rapid_charge();

        battery_conservation
            .enable()
            .ignore()
            .now()
            .expect("battery conservation enable failed");

        rapid_charge
            .enable()
            .ignore()
            .now()
            .expect("rapid charge enable failed");

        assert!(
            rapid_charge
                .enabled()
                .expect("failed to get rapid charge status"),
            "expected rapid charge to be enabled with the ignore handler",
        );

        assert!(
            battery_conservation
                .disabled()
                .expect("failed to get battery conservation status"),
            "expected battery conservation to be disabled with the ignore handler",
        );
    }

    #[test]
    fn test_enable_error() {
        todo!()
    }

    #[test]
    fn test_enable_switch() {
        todo!()
    }

    #[test]
    fn test_disable() {
        todo!()
    }

    #[test]
    fn test_get() {
        todo!()
    }

    #[test]
    fn test_enabled() {
        todo!()
    }

    #[test]
    fn test_disabled() {
        todo!()
    }
}
