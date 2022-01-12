use crate::acpi_call::{self, acpi_call, acpi_call_expect_valid};
use crate::profile::{NewProfile, SystemPerformanceBits, SystemPerformanceParameters};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("`acpi_call` returned conflicting spmo bit ({spmo}) and fcmo bit ({fcmo}) system performance return values (system performance value from fcmo was {spm_fcmo:?}, system performance value from spmo was {spm_spmo:?}) (this shouldn't happen)")]
    MismatchedFcmoSpmo { fcmo: u32, spm_fcmo: SystemPerformanceMode, spmo: u32, spm_spmo: SystemPerformanceMode },

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
    pub const fn from_u32_setter(parameters: &SystemPerformanceParameters, value: u32) -> Option<Self> {
        match value {
            _ if value == parameters.intelligent_cooling => Some(Self::IntelligentCooling),
            _ if value == parameters.extreme_performance => Some(Self::ExtremePerformance),
            _ if value == parameters.battery_saving => Some(Self::BatterySaving),
            _ => None,
        }
    }

    pub const fn from_spmo(bits: &SystemPerformanceBits, spmo: u32) -> Option<Self> {
        match spmo {
            _ if spmo == bits.intelligent_cooling.spmo() => Some(Self::IntelligentCooling),
            _ if spmo == bits.extreme_performance.spmo() => Some(Self::ExtremePerformance),
            _ if spmo == bits.battery_saving.spmo() => Some(Self::BatterySaving),
            _ => None,
        }
    }

    pub const fn from_fcmo(bits: &SystemPerformanceBits, fcmo: u32) -> Option<Self> {
        match fcmo {
            _ if fcmo == bits.intelligent_cooling.fcmo() => Some(Self::IntelligentCooling),
            _ if fcmo == bits.extreme_performance.fcmo() => Some(Self::ExtremePerformance),
            _ if fcmo == bits.battery_saving.fcmo() => Some(Self::BatterySaving),
            _ => None,
        }
    }

    pub const fn spmo(self, bits: &SystemPerformanceBits) -> u32 {
        match self {
            Self::IntelligentCooling => bits.intelligent_cooling.spmo(),
            Self::ExtremePerformance => bits.extreme_performance.spmo(),
            Self::BatterySaving => bits.battery_saving.spmo(),
        }
    }

    pub const fn fcmo(self, bits: &SystemPerformanceBits) -> u32 {
        match self {
            Self::IntelligentCooling => bits.intelligent_cooling.fcmo(),
            Self::ExtremePerformance => bits.extreme_performance.fcmo(),
            Self::BatterySaving => bits.battery_saving.fcmo(),
        }
    }

    pub const fn setter(self, parameters: &SystemPerformanceParameters) -> u32 {
        match self {
            Self::IntelligentCooling => parameters.intelligent_cooling,
            Self::ExtremePerformance => parameters.extreme_performance,
            Self::BatterySaving => parameters.battery_saving,
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SystemPerformanceController<'p> {
    pub profile: &'p NewProfile,
}

impl<'p> SystemPerformanceController<'p> {
    pub const fn new(profile: &'p NewProfile) -> Self {
        Self { profile }
    }

    pub fn set(&self, mode: SystemPerformanceMode) -> acpi_call::Result<()> {
        acpi_call(
            self.profile.system_performance.commands.set.to_string(),
            [mode.setter(&self.profile.system_performance.parameters)],
        )?;

        Ok(())
    }

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

pub fn get() -> Result<SystemPerformanceMode> {
    NewProfile::get().system_performance().get()
}

pub fn set(mode: SystemPerformanceMode) -> acpi_call::Result<()> {
    NewProfile::get().system_performance().set(mode)
}
