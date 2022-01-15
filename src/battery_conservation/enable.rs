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
        controller.context.profile.battery.set_command.to_string(),
        [controller.context.profile.battery.conservation.parameters.enable],
    )?;

    Ok(())
}

/// Enable battery conservation, returning an [`Error::RapidChargeEnabled`] if rapid charge is
/// already enabled.
fn enable_error(controller: &mut BatteryConservationController) -> super::Result<()> {
    if controller.context.profile.rapid_charge().enabled()? {
        Err(super::Error::RapidChargeEnabled)
    } else {
        enable_ignore(controller).map_err(Into::into)
    }
}

/// Enable battery conservation, switching off rapid charge if it is enabled.
fn enable_switch(controller: &mut BatteryConservationController) -> acpi_call::Result<()> {
    let mut rapid_charge = controller.context.profile.rapid_charge();

    if rapid_charge.enabled()? {
        rapid_charge.disable()?;
    }

    enable_ignore(controller)
}
