//! Control the system performance.
//!
//! System performance (modes) are a variety of modes used to control the system performance.

use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::profile::{Profile, SystemPerformanceBits, SystemPerformanceParameters};
use thiserror::Error;

/// Handy wrapper for [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Bad things that could happen when dealing with system performance.
#[derive(Debug, Error)]
pub enum Error {
    /// Mismatched FCMO and SPMO bits. This error should never happen.
    #[error("`acpi_call` returned conflicting spmo bit ({spmo}) and fcmo bit ({fcmo}) system performance return values (system performance value from fcmo was {spm_fcmo:?}, system performance value from spmo was {spm_spmo:?}) (this shouldn't happen)")]
    MismatchedFcmoSpmo {
        /// The mismatched fcmo bit.
        fcmo: u32,

        /// The returned [`SystemPerformanceMode`] from the fcmo bit.
        spm_fcmo: SystemPerformanceMode,

        /// The mismatched spmo bit.
        spmo: u32,

        /// The returned [`SystemPerformanceMode`] from the spmo bit.
        spm_spmo: SystemPerformanceMode,
    },

    /// The system performance mode returned from the a bit was invalid.
    #[error("got invalid system performance mode ({bit}) from `acpi_call`")]
    InvalidSystemPerformanceMode {
        /// The invalid bit.
        bit: u32,
    },

    /// An error occurred when calling `acpi_call`.
    #[error("{error}")]
    AcpiCall {
        /// The underlying error itself.
        #[from]
        error: acpi_call::Error,
    },
}

/// The different system performance modes. Documentation sources can be found
/// [here](https://download.lenovo.com/pccbbs/mobiles_pdf/tp_how_to_use_lenovo_intelligent_cooling_feature.pdf).
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SystemPerformanceMode {
    /// Fan speed and performance are dynamically balanced for better experience.
    IntelligentCooling,

    /// The maximum performance is prioritized, allowing higher temperature and fan speed.
    ExtremePerformance,

    /// Fan speed and performance are lowered to get your computer cooler and quieter, and to get
    /// the best battery life.
    BatterySaving,
}

impl SystemPerformanceMode {
    /// Get system performance mode from a parameter.
    pub const fn from_u32_setter(parameters: &SystemPerformanceParameters, value: u32) -> Option<Self> {
        match value {
            _ if value == parameters.intelligent_cooling => Some(Self::IntelligentCooling),
            _ if value == parameters.extreme_performance => Some(Self::ExtremePerformance),
            _ if value == parameters.battery_saving => Some(Self::BatterySaving),
            _ => None,
        }
    }

    /// Get system performance mode from spmo bit.
    pub const fn from_spmo(bits: &SystemPerformanceBits, spmo: u32) -> Option<Self> {
        match spmo {
            _ if spmo == bits.intelligent_cooling.spmo() => Some(Self::IntelligentCooling),
            _ if spmo == bits.extreme_performance.spmo() => Some(Self::ExtremePerformance),
            _ if spmo == bits.battery_saving.spmo() => Some(Self::BatterySaving),
            _ => None,
        }
    }

    /// Get system performance mode from fcmo bit.
    pub const fn from_fcmo(bits: &SystemPerformanceBits, fcmo: u32) -> Option<Self> {
        match fcmo {
            _ if fcmo == bits.intelligent_cooling.fcmo() => Some(Self::IntelligentCooling),
            _ if fcmo == bits.extreme_performance.fcmo() => Some(Self::ExtremePerformance),
            _ if fcmo == bits.battery_saving.fcmo() => Some(Self::BatterySaving),
            _ => None,
        }
    }

    /// Get the spmo bit of this system performance mode.
    pub const fn spmo(self, bits: &SystemPerformanceBits) -> u32 {
        match self {
            Self::IntelligentCooling => bits.intelligent_cooling.spmo(),
            Self::ExtremePerformance => bits.extreme_performance.spmo(),
            Self::BatterySaving => bits.battery_saving.spmo(),
        }
    }

    /// Get the fcmo bit of this system performance mode.
    pub const fn fcmo(self, bits: &SystemPerformanceBits) -> u32 {
        match self {
            Self::IntelligentCooling => bits.intelligent_cooling.fcmo(),
            Self::ExtremePerformance => bits.extreme_performance.fcmo(),
            Self::BatterySaving => bits.battery_saving.fcmo(),
        }
    }

    /// Get the setter parameter of this system performance mode.
    pub const fn setter(self, parameters: &SystemPerformanceParameters) -> u32 {
        match self {
            Self::IntelligentCooling => parameters.intelligent_cooling,
            Self::ExtremePerformance => parameters.extreme_performance,
            Self::BatterySaving => parameters.battery_saving,
        }
    }
}

/// Controller for the system performance mode.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SystemPerformanceController<'p> {
    /// The reference to the profile.
    pub profile: &'p Profile,
}

impl<'p> SystemPerformanceController<'p> {
    /// Create a new system performance controller.
    pub const fn new(profile: &'p Profile) -> Self {
        Self { profile }
    }

    /// Set the system performance mode to the specified mode.
    pub fn set(&self, mode: SystemPerformanceMode) -> acpi_call::Result<()> {
        acpi_call(
            self.profile.system_performance.commands.set.to_string(),
            [mode.setter(&self.profile.system_performance.parameters)],
        )?;

        Ok(())
    }

    /// Get the system performance mode.
    pub fn get(&self) -> Result<SystemPerformanceMode> {
        let spmo = acpi_call_expect_valid(
                self.profile.system_performance.commands.get_spmo_bit.to_string(),
            [],
        )?;
        let fcmo = acpi_call_expect_valid(
            self.profile.system_performance.commands.get_fcmo_bit.to_string(),
            [],
        )?;

        let spm_spmo = SystemPerformanceMode::from_spmo(&self.profile.system_performance.bits, spmo)
            .ok_or(Error::InvalidSystemPerformanceMode { bit: spmo })?;
        let spm_fcmo = SystemPerformanceMode::from_fcmo(&self.profile.system_performance.bits, fcmo)
            .ok_or(Error::InvalidSystemPerformanceMode { bit: fcmo })?;

        if spm_spmo != spm_fcmo {
            return Err(Error::MismatchedFcmoSpmo { fcmo, spm_fcmo, spmo, spm_spmo })
        };

        // we have proven that system performance mode values are the same at this point, so just
        // return the spmo bit
        Ok(spm_spmo)
    }
}

/// Uses the global profile. See [`SystemPerformanceController::get`] for documentation.
pub fn get() -> Result<SystemPerformanceMode> {
    Profile::get().system_performance().get()
}

/// Uses the global profile. See [`SystemPerformanceController::set`] for documentation.
pub fn set(mode: SystemPerformanceMode) -> acpi_call::Result<()> {
    Profile::get().system_performance().set(mode)
}
