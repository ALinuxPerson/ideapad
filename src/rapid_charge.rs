use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::profile::Profile;
use crate::Handler;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{error}")]
    AcpiCall {
        #[from]
        error: acpi_call::Error,
    },

    #[error("battery conservation is enabled, disable it before enabling rapid charge")]
    BatteryConservationEnabled,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RapidChargeController<'p> {
    pub profile: &'p Profile,
}

impl<'p> RapidChargeController<'p> {
    pub const fn new(profile: &'p Profile) -> Self {
        Self { profile }
    }

    pub fn enable_with_handler(&self, handler: Handler) -> Result<()> {
        match handler {
            Handler::Ignore => self.enable_unchecked().map_err(Into::into),
            Handler::Error => self.enable_strict(),
            Handler::Switch => self.enable().map_err(Into::into),
        }
    }

    pub fn enable_unchecked(&self) -> acpi_call::Result<()> {
        acpi_call(
                self.profile.battery.set_command.to_string(),
            [self.profile.battery.rapid_charge.parameters.enable],
        )?;

        Ok(())
    }

    pub fn enable_strict(&self) -> Result<()> {
        if self.profile.battery_conservation().enabled()? {
            Err(Error::BatteryConservationEnabled)
        } else {
            self.enable_unchecked().map_err(Into::into)
        }
    }

    pub fn enable(&self) -> acpi_call::Result<()> {
        let battery_conservation = self.profile.battery_conservation();

        if battery_conservation.enabled()? {
            battery_conservation.disable()?
        }

        self.enable_unchecked()
    }

    pub fn disable(&self) -> acpi_call::Result<()> {
        acpi_call(
                self.profile.battery.set_command.to_string(),
            [self.profile.battery.rapid_charge.parameters.disable],
        )?;

        Ok(())
    }

    pub fn get(&self) -> acpi_call::Result<bool> {
        let output = acpi_call_expect_valid(
            self.profile.battery.rapid_charge.get_command.to_string(),
            []
        )?;

        Ok(output != 0)
    }

    pub fn enabled(&self) -> acpi_call::Result<bool> {
        self.get()
    }

    pub fn disabled(&self) -> acpi_call::Result<bool> {
        self.get().map(|enabled| !enabled)
    }
}

pub fn enable_with_handler(handler: Handler) -> Result<()> {
    Profile::get().rapid_charge().enable_with_handler(handler)
}

pub fn enable() -> acpi_call::Result<()> {
    Profile::get().rapid_charge().enable()
}

pub fn enable_unchecked() -> acpi_call::Result<()> {
    Profile::get().rapid_charge().enable_unchecked()
}

pub fn enable_strict() -> Result<()> {
    Profile::get().rapid_charge().enable_strict()
}

pub fn disable() -> acpi_call::Result<()> {
    Profile::get().rapid_charge().disable()
}

pub fn get() -> acpi_call::Result<bool> {
    Profile::get().rapid_charge().get()
}

pub fn enabled() -> acpi_call::Result<bool> {
    Profile::get().rapid_charge().enabled()
}

pub fn disabled() -> acpi_call::Result<bool> {
    Profile::get().rapid_charge().disabled()
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    fn test_enable_with_handler() { todo!() }

    #[cfg(test)]
    fn test_enable_unchecked() { todo!() }

    #[cfg(test)]
    fn test_enable_strict() { todo!() }

    #[cfg(test)]
    fn test_enable() { todo!() }

    #[cfg(test)]
    fn test_disable() { todo!() }

    #[cfg(test)]
    fn test_get() { todo!() }

    #[cfg(test)]
    fn test_enabled() { todo!() }

    #[cfg(test)]
    fn test_disabled() { todo!() }
}
