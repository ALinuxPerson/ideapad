use crate::battery_conservation_mode::BatteryConservationModeController;
use crate::rapid_charge::RapidChargeController;
use once_cell::sync::OnceCell;
use parking_lot::{RwLock, RwLockReadGuard};
use smbioslib::SMBiosSystemInformation;
use std::borrow::Cow;
use std::io;
use thiserror::Error;
use crate::SystemPerformanceModeController;

pub type Result<T, E = Error> = std::result::Result<T, E>;

static PROFILE: OnceCell<RwLock<Profile>> = OnceCell::new();

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
pub struct Parameters {
    pub set_to_intelligent_cooling: u32,
    pub set_to_extreme_performance: u32,
    pub set_to_battery_saving: u32,
    pub enable_rapid_charge: u32,
    pub disable_rapid_charge: u32,
    pub enable_battery_conservation: u32,
    pub disable_battery_conservation: u32,
}

impl Parameters {
    pub const SHARED: Self = Self {
        set_to_intelligent_cooling: 0x000FB001,
        set_to_extreme_performance: 0x0012B001,
        set_to_battery_saving: 0x0013B001,
        enable_rapid_charge: 0x07,
        disable_rapid_charge: 0x08,
        enable_battery_conservation: 0x03,
        disable_battery_conservation: 0x05,
    };
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceModeBit {
    pub spmo: u32,
    pub fcmo: u32,
}

impl SystemPerformanceModeBit {
    pub const fn same(value: u32) -> Self {
        Self {
            spmo: value,
            fcmo: value,
        }
    }

    pub const fn is_same(self) -> bool {
        self.spmo == self.fcmo
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceModeBits {
    pub intelligent_cooling: SystemPerformanceModeBit,
    pub extreme_performance: SystemPerformanceModeBit,
    pub battery_saving: SystemPerformanceModeBit,
}

impl SystemPerformanceModeBits {
    pub const SHARED: Self = Self {
        intelligent_cooling: SystemPerformanceModeBit::same(0x0),
        extreme_performance: SystemPerformanceModeBit::same(0x1),
        battery_saving: SystemPerformanceModeBit::same(0x2),
    };
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Profile {
    pub set_system_performance_mode: Cow<'static, str>,
    pub get_system_performance_mode_fcmo: Cow<'static, str>,
    pub get_system_performance_mode_spmo: Cow<'static, str>,
    pub system_performance_mode_bits: SystemPerformanceModeBits,
    pub set_battery_methods: Cow<'static, str>,
    pub get_battery_conservation_mode: Cow<'static, str>,
    pub get_rapid_charge: Cow<'static, str>,
    pub name: Cow<'static, str>,
    pub expected_product_names: Cow<'static, [Cow<'static, str>]>,
    pub parameters: Parameters,
}

#[macro_export]
macro_rules! const_static_profile {
    (Profile {
        set_system_performance_mode: $system_performance_mode_method:expr,
        get_system_performance_mode_fcmo: $get_system_performance_mode_fcmo:expr,
        get_system_performance_mode_spmo: $get_system_performance_mode_spmo:expr,
        system_performance_mode_bits: $system_performance_mode_bits:expr,
        set_battery_methods: $battery_methods:expr,
        get_battery_conservation_mode: $get_battery_conservation_mode:expr,
        get_rapid_charge: $get_rapid_charge:expr,
        name: $name:expr,
        expected_product_names: &[$($expected_product_name:expr,)*],
        parameters: $parameters:expr,
    }) => {
        $crate::profile::Profile {
            set_system_performance_mode: ::std::borrow::Cow::Borrowed($system_performance_mode_method),
            get_system_performance_mode_fcmo: ::std::borrow::Cow::Borrowed($get_system_performance_mode_fcmo),
            get_system_performance_mode_spmo: ::std::borrow::Cow::Borrowed($get_system_performance_mode_spmo),
            system_performance_mode_bits: $system_performance_mode_bits,
            set_battery_methods: ::std::borrow::Cow::Borrowed($battery_methods),
            get_battery_conservation_mode: ::std::borrow::Cow::Borrowed($get_battery_conservation_mode),
            get_rapid_charge: ::std::borrow::Cow::Borrowed($get_rapid_charge),
            name: ::std::borrow::Cow::Borrowed($name),
            expected_product_names: ::std::borrow::Cow::Borrowed(&[$(::std::borrow::Cow::Borrowed($expected_product_name),)*]),
            parameters: $parameters,
        }
    };
}

#[macro_export]
macro_rules! const_dynamic_profile {
    (Profile {
        set_system_performance_mode: $system_performance_mode_method:expr,
        get_system_performance_mode_fcmo: $get_system_performance_mode_fcmo:expr,
        get_system_performance_mode_spmo: $get_system_performance_mode_spmo:expr,
        system_performance_mode_bits: $system_performance_mode_bits:expr,
        set_battery_methods: $battery_methods:expr,
        get_battery_conservation_mode: $get_battery_conservation_mode:expr,
        get_rapid_charge: $get_rapid_charge:expr,
        name: $name:expr,
        expected_product_names: &[$($expected_product_name:expr,)*],
        parameters: $parameters:expr,
    }) => {
        $crate::profile::Profile {
            set_system_performance_mode: ::std::borrow::Cow::Owned($system_performance_mode_method),
            get_system_performance_mode_fcmo: ::std::borrow::Cow::Owned($get_system_performance_mode_fcmo),
            get_system_performance_mode_spmo: ::std::borrow::Cow::Owned($get_system_performance_mode_spmo),
            system_performance_mode_bits: $system_performance_mode_bits,
            set_battery_methods: ::std::borrow::Cow::Owned($battery_methods),
            get_battery_conservation_mode: ::std::borrow::Cow::Owned($get_battery_conservation_mode),
            get_rapid_charge: ::std::borrow::Cow::Owned($get_rapid_charge),
            name: ::std::borrow::Cow::Owned($name),
            expected_product_names: ::std::borrow::Cow::Owned(&[$(::std::borrow::Cow::Owned($expected_product_name),)*]),
            parameters: $parameters,
        }
    };
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ProfileBuilder<SSPM, GSPMF, GSPMS, SBM, GBCM, GRC, N, EPN>
where
    SSPM: Into<Cow<'static, str>>,
    GSPMF: Into<Cow<'static, str>>,
    GSPMS: Into<Cow<'static, str>>,
    SBM: Into<Cow<'static, str>>,
    GBCM: Into<Cow<'static, str>>,
    GRC: Into<Cow<'static, str>>,
    N: Into<Cow<'static, str>>,
    EPN: IntoIterator,
    EPN::Item: Into<Cow<'static, str>>,
{
    pub set_system_performance_mode: SSPM,
    pub get_system_performance_mode_fcmo: GSPMF,
    pub get_system_performance_mode_spmo: GSPMS,
    pub system_performance_mode_bits: SystemPerformanceModeBits,
    pub set_battery_methods: SBM,
    pub get_battery_conservation_mode: GBCM,
    pub get_rapid_charge: GRC,
    pub name: N,
    pub expected_product_names: EPN,
    pub parameters: Parameters,
}

impl<SSPM, GSPMF, GSPMS, SBM, GBCM, GRC, N, EPN>
    ProfileBuilder<SSPM, GSPMF, GSPMS, SBM, GBCM, GRC, N, EPN>
where
    SSPM: Into<Cow<'static, str>>,
    GSPMF: Into<Cow<'static, str>>,
    GSPMS: Into<Cow<'static, str>>,
    SBM: Into<Cow<'static, str>>,
    GBCM: Into<Cow<'static, str>>,
    GRC: Into<Cow<'static, str>>,
    N: Into<Cow<'static, str>>,
    EPN: IntoIterator,
    EPN::Item: Into<Cow<'static, str>>,
{
    pub fn into_profile(self) -> Profile {
        self.into()
    }
}

impl<SSPM, GSPMF, GSPMS, SBM, GBCM, GRC, N, EPN>
    From<ProfileBuilder<SSPM, GSPMF, GSPMS, SBM, GBCM, GRC, N, EPN>> for Profile
where
    SSPM: Into<Cow<'static, str>>,
    GSPMF: Into<Cow<'static, str>>,
    GSPMS: Into<Cow<'static, str>>,
    SBM: Into<Cow<'static, str>>,
    GBCM: Into<Cow<'static, str>>,
    GRC: Into<Cow<'static, str>>,
    N: Into<Cow<'static, str>>,
    EPN: IntoIterator,
    EPN::Item: Into<Cow<'static, str>>,
{
    fn from(builder: ProfileBuilder<SSPM, GSPMF, GSPMS, SBM, GBCM, GRC, N, EPN>) -> Self {
        Profile {
            set_system_performance_mode: builder.set_system_performance_mode.into(),
            get_system_performance_mode_fcmo: builder.get_system_performance_mode_fcmo.into(),
            get_system_performance_mode_spmo: builder.get_system_performance_mode_spmo.into(),
            system_performance_mode_bits: builder.system_performance_mode_bits,
            set_battery_methods: builder.set_battery_methods.into(),
            get_battery_conservation_mode: builder.get_battery_conservation_mode.into(),
            get_rapid_charge: builder.get_rapid_charge.into(),
            name: builder.name.into(),
            expected_product_names: builder
                .expected_product_names
                .into_iter()
                .map(|x| x.into())
                .collect(),
            parameters: builder.parameters,
        }
    }
}

impl Profile {
    pub const IDEAPAD_15IIL05: Self = const_static_profile! {
        Profile {
            set_system_performance_mode: r#"\_SB.PCI0.LPCB.EC0.VPC0.DYTC"#,
            get_system_performance_mode_fcmo: r#"\_SB.PCI0.LPCB.EC0.FCMO"#,
            get_system_performance_mode_spmo: r#"\_SB.PCI0.LPCB.EC0.SPMO"#,
            system_performance_mode_bits: SystemPerformanceModeBits::SHARED,
            set_battery_methods: r#"\_SB.PCI0.LPCB.EC0.VPC0.SBMC"#,
            get_battery_conservation_mode: r#"\_SB.PCI0.LPCB.EC0.BTSM"#,
            get_rapid_charge: r#"\_SB.PCI0.LPCB.EC0.QCHO"#,
            name: "IDEAPAD_15IIL05",
            expected_product_names: &["81YK", ],
            parameters: Parameters::SHARED,
        }
    };
    pub const IDEAPAD_15_AMD: Self = const_static_profile! {
        Profile {
            set_system_performance_mode: r#"\_SB.PCI0.LPC0.EC0.VPC0.DYTC"#,
            get_system_performance_mode_fcmo: r#"\_SB.PCI0.LPC0.EC0.FCMO"#,
            get_system_performance_mode_spmo: r#"\_SB.PCI0.LPC0.EC0.SPMO"#,
            system_performance_mode_bits: SystemPerformanceModeBits::SHARED,
            set_battery_methods: r#"\_SB.PCI0.LPC0.EC0.VPC0.SBMC"#,
            get_battery_conservation_mode: r#"\_SB.PCI0.LPC0.EC0.BTSM"#,
            get_rapid_charge: r#"\_SB.PCI0.LPC0.EC0.QCHO"#,
            name: "IDEAPAD_15_AMD",
            expected_product_names: &[
                "81YQ", // 15ARE05
                "81YM", // 14ARE05
            ],
            parameters: Parameters::SHARED,
        }
    };
    pub const SEARCH_PATH: &'static [Self] = &[Self::IDEAPAD_15IIL05, Self::IDEAPAD_15_AMD];

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
        match PROFILE.get() {
            Some(profile) => Ok(profile.read()),
            None => {
                let profile = Self::find_with_search_path(search_path)?;
                let _ = PROFILE.set(RwLock::new(profile));
                Ok(PROFILE.get().expect("PROFILE should be set").read())
            }
        }
    }

    pub fn initialize_with_profile(profile: Self) -> RwLockReadGuard<'static, Self> {
        match PROFILE.get() {
            Some(profile) => profile.read(),
            None => {
                let _ = PROFILE.set(RwLock::new(profile));
                PROFILE.get().expect("PROFILE should be set").read()
            }
        }
    }

    pub fn get() -> RwLockReadGuard<'static, Self> {
        PROFILE.get()
            .expect("profile not initialized (tip: initialize it with the variety of methods in `Profile` or use `ideapad::initialize()` for defaults)")
            .read()
    }

    pub fn set(this: Self) {
        if let Err(this) = PROFILE.set(RwLock::new(this)) {
            let this = this.into_inner();
            let mut global_profile = PROFILE
                .get()
                .expect(
                    "profile not initialized but why does `PROFILE.set(...)` return `Err(...)`?",
                )
                .write();

            *global_profile = this
        }
    }

    pub fn battery_conservation_mode(&self) -> BatteryConservationModeController {
        BatteryConservationModeController::new(self)
    }

    pub fn rapid_charge(&self) -> RapidChargeController {
        RapidChargeController::new(self)
    }

    pub fn system_performance_mode(&self) -> SystemPerformanceModeController {
        SystemPerformanceModeController::new(self)
    }
}

mod tests {

}