use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::profile::{Parameters, OldProfile, SystemPerformanceModeBits};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("got mismatched fcmo bits ({fcmo}) and spmo bits ({spmo}) from `acpi_call` (this shouldn't happen)")]
    MismatchedFcmoSpmo { fcmo: u32, spmo: u32 },

    #[error("got invalid system performance mode ({bit}) from `acpi_call`")]
    InvalidSystemPerformanceMode { bit: u32 },

    #[error("{error}")]
    AcpiCall {
        #[from]
        error: acpi_call::Error,
    },
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SystemPerformanceMode {
    IntelligentCooling,
    ExtremePerformance,
    BatterySaving,
}

impl SystemPerformanceMode {
    pub const fn from_u32_setter(parameters: &Parameters, value: u32) -> Option<Self> {
        match value {
            _ if value == parameters.set_to_intelligent_cooling => Some(Self::IntelligentCooling),
            _ if value == parameters.set_to_extreme_performance => Some(Self::ExtremePerformance),
            _ if value == parameters.set_to_battery_saving => Some(Self::BatterySaving),
            _ => None,
        }
    }

    pub const fn from_spmo(bits: &SystemPerformanceModeBits, spmo: u32) -> Option<Self> {
        match spmo {
            _ if spmo == bits.intelligent_cooling.spmo => Some(Self::IntelligentCooling),
            _ if spmo == bits.extreme_performance.spmo => Some(Self::ExtremePerformance),
            _ if spmo == bits.battery_saving.spmo => Some(Self::BatterySaving),
            _ => None,
        }
    }

    pub const fn from_fcmo(bits: &SystemPerformanceModeBits, fcmo: u32) -> Option<Self> {
        match fcmo {
            _ if fcmo == bits.intelligent_cooling.fcmo => Some(Self::IntelligentCooling),
            _ if fcmo == bits.extreme_performance.fcmo => Some(Self::ExtremePerformance),
            _ if fcmo == bits.battery_saving.fcmo => Some(Self::BatterySaving),
            _ => None,
        }
    }

    pub const fn spmo(self, bits: &SystemPerformanceModeBits) -> u32 {
        match self {
            Self::IntelligentCooling => bits.intelligent_cooling.spmo,
            Self::ExtremePerformance => bits.extreme_performance.spmo,
            Self::BatterySaving => bits.battery_saving.spmo,
        }
    }

    pub const fn fcmo(self, bits: &SystemPerformanceModeBits) -> u32 {
        match self {
            Self::IntelligentCooling => bits.intelligent_cooling.fcmo,
            Self::ExtremePerformance => bits.extreme_performance.fcmo,
            Self::BatterySaving => bits.battery_saving.fcmo,
        }
    }

    pub const fn setter(self, parameters: &Parameters) -> u32 {
        match self {
            Self::IntelligentCooling => parameters.set_to_intelligent_cooling,
            Self::ExtremePerformance => parameters.set_to_extreme_performance,
            Self::BatterySaving => parameters.set_to_battery_saving,
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SystemPerformanceController<'p> {
    pub profile: &'p OldProfile,
}

impl<'p> SystemPerformanceController<'p> {
    pub fn new(profile: &'p OldProfile) -> Self {
        Self { profile }
    }

    pub fn set(&self, mode: SystemPerformanceMode) -> acpi_call::Result<()> {
        acpi_call(
            self.profile.set_system_performance_mode.to_string(),
            [mode.setter(&self.profile.parameters)],
        )?;

        Ok(())
    }

    pub fn get(&self) -> Result<SystemPerformanceMode> {
        let spmo = acpi_call_expect_valid(
            self.profile.get_system_performance_mode_spmo.to_string(),
            [],
        )?;
        let fcmo = acpi_call_expect_valid(
            self.profile.get_system_performance_mode_fcmo.to_string(),
            [],
        )?;

        if spmo != fcmo {
            return Err(Error::MismatchedFcmoSpmo { fcmo, spmo });
        }

        SystemPerformanceMode::from_spmo(&self.profile.system_performance_mode_bits, spmo)
            .ok_or(Error::InvalidSystemPerformanceMode { bit: spmo })
    }
}

pub fn get() -> Result<SystemPerformanceMode> {
    OldProfile::get().system_performance_mode().get()
}

pub fn set(mode: SystemPerformanceMode) -> acpi_call::Result<()> {
    OldProfile::get().system_performance_mode().set(mode)
}
