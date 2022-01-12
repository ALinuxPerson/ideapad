use crate::battery_conservation::BatteryConservationController;
use crate::rapid_charge::RapidChargeController;
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard};
use smbioslib::SMBiosSystemInformation;
use std::borrow::Cow;
use std::io;
use thiserror::Error;
use crate::SystemPerformanceController;

pub type Result<T, E = Error> = std::result::Result<T, E>;

static NEW_PROFILE: OnceCell<RwLock<Profile>> = OnceCell::new();

#[derive(Debug, Error)]
pub enum Error {
    #[error("{error}")]
    Io {
        #[from]
        error: io::Error,
    },

    #[error("unable to find system information from smbios")]
    UnableToFindSystemInformation,

    #[error("no valid profiles were found in the search path")]
    NoValidProfileInSearchPath,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BitInner {
    Same(u32),
    Different {
        spmo: u32,
        fcmo: u32,
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bit(BitInner);

impl Bit {
    pub const fn same(value: u32) -> Self {
        Self(BitInner::Same(value))
    }

    pub const fn different(spmo: u32, fcmo: u32) -> Self {
        if spmo == fcmo {
            Self(BitInner::Same(spmo))
        } else {
            Self(BitInner::Different { spmo, fcmo })
        }
    }

    pub const fn inner(&self) -> BitInner {
        self.0
    }

    pub const fn spmo(&self) -> u32 {
        match self.0 {
            BitInner::Same(value) => value,
            BitInner::Different { spmo, .. } => spmo,
        }
    }

    pub const fn fcmo(&self) -> u32 {
        match self.0 {
            BitInner::Same(value) => value,
            BitInner::Different { fcmo, .. } => fcmo,
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceCommands {
    pub set: Cow<'static, str>,
    pub get_fcmo_bit: Cow<'static, str>,
    pub get_spmo_bit: Cow<'static, str>,
}

impl SystemPerformanceCommands {
    pub const fn r#static(set: &'static str, get_fcmo_bit: &'static str, get_spmo_bit: &'static str) -> Self {
        Self {
            set: Cow::Borrowed(set),
            get_fcmo_bit: Cow::Borrowed(get_fcmo_bit),
            get_spmo_bit: Cow::Borrowed(get_spmo_bit),
        }
    }

    pub const fn dynamic(set: String, get_fcmo_bit: String, get_spmo_bit: String) -> Self {
        Self {
            set: Cow::Owned(set),
            get_fcmo_bit: Cow::Owned(get_fcmo_bit),
            get_spmo_bit: Cow::Owned(get_spmo_bit),
        }
    }

    pub fn new(
        set: impl Into<Cow<'static, str>>,
        get_fcmo_bit: impl Into<Cow<'static, str>>,
        get_spmo_bit: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            set: set.into(),
            get_fcmo_bit: get_fcmo_bit.into(),
            get_spmo_bit: get_spmo_bit.into(),
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceParameters {
    pub intelligent_cooling: u32,
    pub extreme_performance: u32,
    pub battery_saving: u32,
}

impl SystemPerformanceParameters {
    pub const SHARED: Self = Self {
        intelligent_cooling: 0x000FB001,
        extreme_performance: 0x0012B001,
        battery_saving: 0x0013B001,
    };

    pub const fn new(intelligent_cooling: u32, extreme_performance: u32, battery_saving: u32) -> Self {
        Self {
            intelligent_cooling,
            extreme_performance,
            battery_saving,
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceBits {
    pub intelligent_cooling: Bit,
    pub extreme_performance: Bit,
    pub battery_saving: Bit,
}

impl SystemPerformanceBits {
    pub const SHARED: Self = Self {
        intelligent_cooling: Bit::same(0x0),
        extreme_performance: Bit::same(0x1),
        battery_saving: Bit::same(0x2),
    };

    pub const fn new(
        intelligent_cooling: Bit,
        extreme_performance: Bit,
        battery_saving: Bit,
    ) -> Self {
        Self {
            intelligent_cooling,
            extreme_performance,
            battery_saving,
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformance {
    pub commands: SystemPerformanceCommands,
    pub bits: SystemPerformanceBits,
    pub parameters: SystemPerformanceParameters,
}

impl SystemPerformance {
    pub const fn new(
        commands: SystemPerformanceCommands,
        bits: SystemPerformanceBits,
        parameters: SystemPerformanceParameters,
    ) -> Self {
        Self {
            commands,
            bits,
            parameters,
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Battery {
    pub set_command: Cow<'static, str>,
    pub conservation: SharedBatteryConfiguration,
    pub rapid_charge: SharedBatteryConfiguration,
}

impl Battery {
    pub const fn r#static(set_command: &'static str, conservation: SharedBatteryConfiguration, rapid_charge: SharedBatteryConfiguration) -> Self {
        Self {
            set_command: Cow::Borrowed(set_command),
            conservation,
            rapid_charge,
        }
    }

    pub const fn dynamic(set_command: String, conservation: SharedBatteryConfiguration, rapid_charge: SharedBatteryConfiguration) -> Self {
        Self {
            set_command: Cow::Owned(set_command),
            conservation,
            rapid_charge,
        }
    }

    pub fn new(
        set_command: impl Into<Cow<'static, str>>,
        conservation: SharedBatteryConfiguration,
        rapid_charge: SharedBatteryConfiguration,
    ) -> Self {
        Self {
            set_command: set_command.into(),
            conservation,
            rapid_charge,
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SharedBatteryConfigurationParameters {
    pub enable: u32,
    pub disable: u32,
}

impl SharedBatteryConfigurationParameters {
    pub const CONSERVATION_SHARED: Self = Self {
        enable: 0x03,
        disable: 0x05,
    };
    pub const RAPID_CHARGE_SHARED: Self = Self {
        enable: 0x07,
        disable: 0x08,
    };

    pub const fn new(enable: u32, disable: u32) -> Self {
        Self {
            enable,
            disable,
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SharedBatteryConfiguration {
    pub get_command: Cow<'static, str>,
    pub parameters: SharedBatteryConfigurationParameters,
}

impl SharedBatteryConfiguration {
    pub const fn r#static(get_command: &'static str, parameters: SharedBatteryConfigurationParameters) -> Self {
        Self {
            get_command: Cow::Borrowed(get_command),
            parameters,
        }
    }

    pub const fn dynamic(get_command: String, parameters: SharedBatteryConfigurationParameters) -> Self {
        Self {
            get_command: Cow::Owned(get_command),
            parameters,
        }
    }

    pub fn new(
        get_command: impl Into<Cow<'static, str>>,
        parameters: SharedBatteryConfigurationParameters,
    ) -> Self {
        Self {
            get_command: get_command.into(),
            parameters,
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Profile {
    pub name: Cow<'static, str>,
    pub expected_product_names: Cow<'static, [Cow<'static, str>]>,
    pub system_performance: SystemPerformance,
    pub battery: Battery,
}

impl Profile {
    pub const IDEAPAD_15IIL05: Self = Self::r#static(
        "IDEAPAD_15IIL05",
        borrowed_cow_array!["81YK"],
        SystemPerformance::new(
            SystemPerformanceCommands::r#static(
                r#"\_SB.PCI0.LPCB.EC0.VPC0.DYTC"#,
                r#"\_SB.PCI0.LPCB.EC0.FCMO"#,
               r#"\_SB.PCI0.LPCB.EC0.SPMO"#,
            ),
            SystemPerformanceBits::SHARED,
            SystemPerformanceParameters::SHARED,
        ),
        Battery::r#static(
            r#"\_SB.PCI0.LPCB.EC0.VPC0.SBMC"#,
            SharedBatteryConfiguration::r#static(
                r#"\_SB.PCI0.LPCB.EC0.BTSM"#,
                SharedBatteryConfigurationParameters::CONSERVATION_SHARED,
            ),
            SharedBatteryConfiguration::r#static(
                r#"\_SB.PCI0.LPCB.EC0.QCHO"#,
                SharedBatteryConfigurationParameters::RAPID_CHARGE_SHARED,
            )
        )
    );
    pub const IDEAPAD_AMD: Self = Self::r#static(
        "IDEAPAD_AMD",
        borrowed_cow_array!["81YQ", "81YM"],
        SystemPerformance::new(
            SystemPerformanceCommands::r#static(
                r#"\_SB.PCI0.LPC0.EC0.VPC0.DYTC"#,
                r#"\_SB.PCI0.LPC0.EC0.FCMO"#,
               r#"\_SB.PCI0.LPC0.EC0.SPMO"#,
            ),
            SystemPerformanceBits::SHARED,
            SystemPerformanceParameters::SHARED,
        ),
        Battery::r#static(
            r#"\_SB.PCI0.LPC0.EC0.VPC0.SBMC"#,
            SharedBatteryConfiguration::r#static(
                r#"\_SB.PCI0.LPC0.EC0.BTSM"#,
                SharedBatteryConfigurationParameters::CONSERVATION_SHARED,
            ),
            SharedBatteryConfiguration::r#static(
                r#"\_SB.PCI0.LPC0.EC0.QCHO"#,
                SharedBatteryConfigurationParameters::RAPID_CHARGE_SHARED,
            )
        )
    );

    pub const fn r#static(
        name: &'static str,
        expected_product_names: &'static [Cow<'static, str>],
        system_performance: SystemPerformance,
        battery: Battery,
    ) -> Self {
        Self {
            name: Cow::Borrowed(name),
            expected_product_names: Cow::Borrowed(expected_product_names),
            system_performance,
            battery,
        }
    }

    pub const fn dynamic(
        name: String,
        expected_product_names: Vec<Cow<'static, str>>,
        system_performance: SystemPerformance,
        battery: Battery,
    ) -> Self {
        Self {
            name: Cow::Owned(name),
            expected_product_names: Cow::Owned(expected_product_names),
            system_performance,
            battery,
        }
    }

    pub fn new(
        name: impl Into<Cow<'static, str>>,
        expected_product_names: impl IntoIterator<Item = impl Into<Cow<'static, str>>>,
        system_performance: SystemPerformance,
        battery: Battery,
    ) -> Self {
        Self {
            name: name.into(),
            expected_product_names: Cow::Owned(
                expected_product_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect(),
            ),
            system_performance,
            battery,
        }
    }

    pub const SEARCH_PATH: &'static [Self] = &[Self::IDEAPAD_15IIL05, Self::IDEAPAD_AMD];

    pub fn find() -> Result<Self> {
        Self::find_with_search_path(Self::SEARCH_PATH.iter().cloned())
    }

    pub fn find_with_search_path(search_path: impl IntoIterator<Item = Self>) -> Result<Self> {
        let product_name = smbioslib::table_load_from_device()?
            .find_map(|system: SMBiosSystemInformation| system.product_name())
            .ok_or(Error::UnableToFindSystemInformation)?;

        search_path
            .into_iter()
            .find(|profile| {
                profile
                    .expected_product_names
                    .contains(&Cow::Borrowed(product_name.as_str()))
            })
            .ok_or(Error::NoValidProfileInSearchPath)
    }

    pub fn auto_detect() -> Result<RwLockReadGuard<'static, Self>> {
        Self::initialize_with_search_path(Self::SEARCH_PATH.iter().cloned())
    }

    pub fn initialize_with_search_path(
        search_path: impl IntoIterator<Item = Self>,
    ) -> Result<RwLockReadGuard<'static, Self>> {
        match NEW_PROFILE.get() {
            Some(profile) => Ok(profile.read()),
            None => {
                let profile = Self::find_with_search_path(search_path)?;
                let _ = NEW_PROFILE.set(RwLock::new(profile));
                Ok(NEW_PROFILE.get().expect("PROFILE should be set").read())
            }
        }
    }

    pub fn initialize_with_profile(profile: Self) -> RwLockReadGuard<'static, Self> {
        match NEW_PROFILE.get() {
            Some(profile) => profile.read(),
            None => {
                let _ = NEW_PROFILE.set(RwLock::new(profile));
                NEW_PROFILE.get().expect("PROFILE should be set").read()
            }
        }
    }

    pub fn get() -> RwLockReadGuard<'static, Self> {
        NEW_PROFILE.get()
            .expect("profile not initialized (tip: initialize it with the variety of methods in `Profile` or use `ideapad::initialize()` for defaults)")
            .read()
    }

    pub fn set(this: Self) {
        if let Err(this) = NEW_PROFILE.set(RwLock::new(this)) {
            let this = this.into_inner();
            let mut global_profile = NEW_PROFILE
                .get()
                .expect(
                    "profile not initialized but why does `PROFILE.set(...)` return `Err(...)`?",
                )
                .write();

            *global_profile = this
        }
    }

    pub const fn battery_conservation(&self) -> BatteryConservationController {
        BatteryConservationController::new(self)
    }

    pub const fn rapid_charge(&self) -> RapidChargeController {
        RapidChargeController::new(self)
    }

    pub const fn system_performance(&self) -> SystemPerformanceController {
        SystemPerformanceController::new(self)
    }
}

mod tests {

}