//! Control battery conservation mode.
//!
//! Battery conservation mode is a mode found in Ideapad laptops which limits the battery's maximum
//! level to 60%. However, if you charge your battery above 60% with battery conservation mode
//! disabled then enable it, the battery level will be capped at the level you enabled battery
//! conservation mode at. For example, if you charge your battery to 80% and then enable battery
//! conservation mode, the battery level will be capped at 80%.
pub mod enable {
    mod private {
        pub trait Sealed {}
    }

    use crate::{BatteryConservationController, Handler};
    use crate::acpi_call::{acpi_call, self};
    use crate::battery_conservation::BatteryConservationEnableGuard;

    /// A stage for the battery conservation enable builder.
    pub trait Stage: private::Sealed {}

    /// The first stage.
    ///
    /// This stage is where you specify the handler.
    pub struct Begin {
        _priv: ()
    }

    impl Stage for Begin {}
    impl private::Sealed for Begin {}

    /// The second stage.
    ///
    /// This stage is where you call the specified method you want, either create an enable guard or
    /// enable immediately.
    pub struct Call {
        handler: Handler,
    }

    impl Stage for Call {}
    impl private::Sealed for Call {}

    /// Builder for enabling battery conservation.
    ///
    /// This builder is seperated into [`Stage`]s; see their documentation for details.
    pub struct EnableBatteryConservationBuilder<'bc, 'p, S: Stage> {
        /// A mutable reference to the battery conservation controller.
        pub controller: &'bc mut BatteryConservationController<'p>,
        stage: S,
    }

    impl<'bc, 'p> EnableBatteryConservationBuilder<'bc, 'p, Begin> {
        /// Create a new builder for enabling battery conservation.
        pub fn new(controller: &'bc mut BatteryConservationController<'p>) -> Self {
            Self { controller, stage: Begin { _priv: () } }
        }

        /// Enable battery conservation with the specified [`Handler`].
        ///
        /// For more information on what do the [`Handler`]s mean, see the [`Handler`] documentation.
        pub fn handler(self, handler: Handler) -> EnableBatteryConservationBuilder<'bc, 'p, Call> {
            EnableBatteryConservationBuilder {
                controller: self.controller,
                stage: Call { handler }
            }
        }

        /// Enable battery conservation, ignoring if rapid charge is already enabled.
        ///
        /// # Note
        /// Using this could drain your battery unnecessarily if rapid charge is enabled. Be
        /// careful!
        ///
        /// # Quirks
        /// It seems like, that at least for the Ideapad 15IIL05, that battery conservation switches
        /// itself off when you enable battery conservation first **then** enable rapid charging.
        ///
        /// This is easily bypassed though by just re-enabling battery conservation afterwards.
        pub fn ignore(self) -> EnableBatteryConservationBuilder<'bc, 'p, Call> {
            self.handler(Handler::Ignore)
        }

        /// Enable battery conservation, returning an [`Error::RapidChargeEnabled`] if rapid charge is
        /// already enabled.
        pub fn error(self) -> EnableBatteryConservationBuilder<'bc, 'p, Call> {
            self.handler(Handler::Error)
        }

        /// Enable battery conservation, switching off rapid charge if it is enabled.
        ///
        /// # Quirks
        /// It seems like, that at least for the Ideapad 15IIL05, it does some form of this
        /// automatically. For more information, see [`Self::ignore`].
        pub fn switch(self) -> EnableBatteryConservationBuilder<'bc, 'p, Call> {
            self.handler(Handler::Switch)
        }
    }

    impl<'bc, 'p> EnableBatteryConservationBuilder<'bc, 'p, Call> {
        /// Get the handler specified.
        pub const fn handler(&self) -> Handler {
            self.stage.handler
        }

        /// Consume the builder to make a guard out of it.
        pub fn guard(self) -> super::Result<BatteryConservationEnableGuard<'bc, 'p>> {
            BatteryConservationEnableGuard::handler(self.controller, self.handler())
        }

        /// Consume the builder to enable battery conservation with the specified handler from the
        /// last stage.
        pub fn now(self) -> super::Result<()> {
            match self.handler() {
                Handler::Ignore => enable_ignore(self.controller).map_err(Into::into),
                Handler::Error => enable_error(self.controller),
                Handler::Switch => enable_switch(self.controller).map_err(Into::into),
            }
        }
    }

    fn enable_ignore(controller: &mut BatteryConservationController) -> acpi_call::Result<()> {
        acpi_call(
            controller.profile.battery.set_command.to_string(),
            [controller.profile.battery.conservation.parameters.enable],
        )?;

        Ok(())
    }

    /// Enable battery conservation, returning an [`Error::RapidChargeEnabled`] if rapid charge is
    /// already enabled.
    fn enable_error(controller: &mut BatteryConservationController) -> super::Result<()> {
        if controller.profile.rapid_charge().enabled()? {
            Err(super::Error::RapidChargeEnabled)
        } else {
            enable_ignore(controller).map_err(Into::into)
        }
    }

    /// Enable battery conservation, switching off rapid charge if it is enabled.
    fn enable_switch(controller: &mut BatteryConservationController) -> acpi_call::Result<()> {
        let mut rapid_charge = controller.profile.rapid_charge();

        if rapid_charge.enabled()? {
            rapid_charge.disable()?;
        }

        enable_ignore(controller)
    }
}

pub use enable::EnableBatteryConservationBuilder;
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

/// "Guarantees" that the battery conservation mode is enabled for the scope.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[must_use]
pub struct BatteryConservationEnableGuard<'bc, 'p> {
    controller: &'bc mut BatteryConservationController<'p>,
}

impl<'bc, 'p> BatteryConservationEnableGuard<'bc, 'p> {
    /// Enable battery conservation mode for the scope with the specified handler.
    pub fn handler(
        controller: &'bc mut BatteryConservationController<'p>,
        handler: Handler,
    ) -> Result<Self> {
        controller.enable_with_handler(handler)?;

        Ok(Self { controller })
    }
}

impl<'bc, 'p> Drop for BatteryConservationEnableGuard<'bc, 'p> {
    fn drop(&mut self) {
        crate::fallible_drop_strategy::handle_error(|| self.controller.disable())
    }
}

/// "Guarantees" that the battery conservation mode is disabled for the scope.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[must_use]
pub struct BatteryConservationDisableGuard<'bc, 'p> {
    controller: &'bc mut BatteryConservationController<'p>,
    handler: Handler,
}

impl<'bc, 'p> BatteryConservationDisableGuard<'bc, 'p> {
    /// Disable battery conservation mode for the scope.
    pub fn new(
        controller: &'bc mut BatteryConservationController<'p>,
        handler: Handler,
    ) -> acpi_call::Result<Self> {
        controller.disable()?;

        Ok(Self {
            controller,
            handler,
        })
    }
}

impl<'bc, 'p> Drop for BatteryConservationDisableGuard<'bc, 'p> {
    fn drop(&mut self) {
        crate::fallible_drop_strategy::handle_error(|| {
            self.controller.enable_with_handler(self.handler)
        })
    }
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

    /// Builder for enabling battery conservation.
    pub fn enable<'bc>(&'bc mut self) -> EnableBatteryConservationBuilder<'bc, 'p, enable::Begin> {
        EnableBatteryConservationBuilder::new(self)
    }

    /// Enable battery conservation with the specified [`Handler`].
    ///
    /// For more information on what do the [`Handler`]s mean, see the [`Handler`] documentation.
    pub fn enable_with_handler(&mut self, handler: Handler) -> Result<()> {
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
    pub fn enable_ignore(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.profile.battery.set_command.to_string(),
            [self.profile.battery.conservation.parameters.enable],
        )?;

        Ok(())
    }

    /// Enable battery conservation, returning an [`Error::RapidChargeEnabled`] if rapid charge is
    /// already enabled.
    pub fn enable_error(&mut self) -> Result<()> {
        if self.profile.rapid_charge().enabled()? {
            Err(Error::RapidChargeEnabled)
        } else {
            self.enable_ignore().map_err(Into::into)
        }
    }

    /// Enable battery conservation, switching off rapid charge if it is enabled.
    pub fn enable_switch(&mut self) -> acpi_call::Result<()> {
        let mut rapid_charge = self.profile.rapid_charge();

        if rapid_charge.enabled()? {
            rapid_charge.disable()?;
        }

        self.enable_ignore()
    }

    /// Disable battery conservation.
    pub fn disable(&mut self) -> acpi_call::Result<()> {
        acpi_call(
            self.profile.battery.set_command.to_string(),
            [self.profile.battery.conservation.parameters.disable],
        )?;

        Ok(())
    }

    /// Get the battery conservation status.
    pub fn get(&self) -> acpi_call::Result<bool> {
        let output = acpi_call_expect_valid(
            self.profile.battery.conservation.get_command.to_string(),
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

    /// Ensures that the battery conservation mode is enabled for this scope.
    pub fn enable_guard<'bc>(
        &'bc mut self,
        handler: Handler,
    ) -> Result<BatteryConservationEnableGuard<'bc, 'p>> {
        BatteryConservationEnableGuard::handler(self, handler)
    }

    /// Ensures that the battery conservation mode is disabled for this scope.
    pub fn disable_guard<'bc>(
        &'bc mut self,
        handler: Handler,
    ) -> acpi_call::Result<BatteryConservationDisableGuard<'bc, 'p>> {
        BatteryConservationDisableGuard::new(self, handler)
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
    Profile::get().battery_conservation().enable_ignore()
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
    use crate::{battery_conservation, rapid_charge, Handler};

    #[test]
    #[serial]
    fn test_enable_with_handler() {
        let profile = crate::initialize().expect("initialization failed");
        let mut battery_conservation = profile.battery_conservation();
        let mut rapid_charge = profile.rapid_charge();

        // set up our scenario here
        battery_conservation
            .enable()
            .handler(Handler::Ignore)
            .now()
            .expect("failed to enable battery conservation");

        // let's test first with ignorance
        rapid_charge.enable_with_handler(Handler::Ignore).expect("rapid charge enable failed");

        assert!(
            rapid_charge.enabled().expect("failed to get rapid charge status"),
            "expected rapid charge to be enabled with the ignore handler",
        );

        // TIL ideapad laptops already have a built in mechanism to switch off rapid charging when
        // trying to enable battery conservation, albeit this is easily bypassed by just switching
        // on battery conservation again afterwards, sooo we still need the switch handler
        assert!(
            battery_conservation.disabled().expect("failed to get battery conservation status"),
            "expected battery conservation to be disabled with the ignore handler",
        );

        // now let's test with an error handler
        battery_conservation.enable()
            .handler(Handler::Ignore)
            .now()
            .expect("failed to enable battery conservation");

        let error = rapid_charge.enable_with_handler(Handler::Error)
            .expect_err("rapid charge enable succeeded");
        assert!(matches!(
            error,
            rapid_charge::Error::BatteryConservationEnabled
        ));
        assert!(battery_conservation.enabled().expect("failed to get battery conservation status"));

        // now let's test with a switch handler
        rapid_charge.enable_with_handler(Handler::Switch).expect("rapid charge enable failed");
        assert!(rapid_charge.enabled().expect("failed to get rapid charge status"));
        assert!(
            battery_conservation.disabled().expect("failed to get battery conservation status")
        );
    }

    #[test]
    #[serial]
    fn test_enable_ignore() {
        let profile = crate::initialize().expect("initialization failed");
        let mut battery_conservation = profile.battery_conservation();
        let mut rapid_charge = profile.rapid_charge();

        battery_conservation.enable().ignore().now().expect("battery conservation enable failed");
        rapid_charge.enable_ignore().expect("rapid charge enable failed");

        assert!(
            rapid_charge.enabled().expect("failed to get rapid charge status"),
            "expected rapid charge to be enabled with the ignore handler",
        );

        assert!(
            battery_conservation.disabled().expect("failed to get battery conservation status"),
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
