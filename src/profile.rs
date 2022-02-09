//! An abstraction which allows this crate to be used on multiple Ideapad models.

use smbioslib::SMBiosSystemInformation;
use std::borrow::Cow;
use std::io;
use thiserror::Error;

/// Handy wrapper for [`enum@Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Bad things which could happen when dealing with [`Profile`]s.
#[derive(Debug, Error)]
pub enum Error {
    /// A generic IO error occurred.
    #[error("{error}")]
    Io {
        /// The underlying IO error.
        #[from]
        error: io::Error,
    },

    /// Unable to get or find the system information from the SMBIOS.
    #[error("unable to find system information from smbios")]
    UnableToFindSystemInformation,

    /// No valid profile was found in the specified search path.
    #[error("no valid profiles were found in the search path")]
    NoValidProfileInSearchPath,
}

/// Actual values of [`Bit`]. It is not guaranteed that [`Self::Different`] would actually be
/// different values; this is why [`Bit`] wraps this type.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BitInner {
    /// Same bits.
    Same(u32),

    /// (not guaranteed to be) different bits.
    Different {
        /// The SPMO bit.
        spmo: u32,

        /// The FCMO bit.
        fcmo: u32,
    },
}

/// Represents an spmo and fcmo bit.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bit(BitInner);

impl Bit {
    /// Create a new bit with the same spmo and fcmo bits.
    pub const fn same(value: u32) -> Self {
        Self::from_inner(BitInner::Same(value))
    }

    /// Create a new bit with different spmo and fcmo bits. If the spmo and fcmo bits are the same,
    /// it will use the same bit.
    pub const fn different(spmo: u32, fcmo: u32) -> Self {
        Self::from_inner(BitInner::Different { spmo, fcmo })
    }

    /// Create a new bit from its inner value.
    pub const fn from_inner(inner: BitInner) -> Self {
        match inner {
            BitInner::Different { spmo, fcmo } if spmo == fcmo => Self::same(spmo),
            _ => Self(inner),
        }
    }

    /// Get the inner value of this bit.
    pub const fn inner(&self) -> BitInner {
        self.0
    }

    /// Get the spmo bit. If same, it will return that bit.
    pub const fn spmo(&self) -> u32 {
        match self.0 {
            BitInner::Same(value) => value,
            BitInner::Different { spmo, .. } => spmo,
        }
    }

    /// Get the fcmo bit. If same, it will return that bit.
    pub const fn fcmo(&self) -> u32 {
        match self.0 {
            BitInner::Same(value) => value,
            BitInner::Different { fcmo, .. } => fcmo,
        }
    }
}

/// Variety of commands which could be used to for system performance.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceCommands {
    /// Set command.
    pub set: Cow<'static, str>,

    /// Get FCMO bit command.
    pub get_fcmo_bit: Cow<'static, str>,

    /// Get SPMO bit command.
    pub get_spmo_bit: Cow<'static, str>,
}

impl SystemPerformanceCommands {
    /// Create a new set of commands which uses stack allocated variants of types which could be
    /// constructed at compile time.
    pub const fn r#static(
        set: &'static str,
        get_fcmo_bit: &'static str,
        get_spmo_bit: &'static str,
    ) -> Self {
        Self {
            set: Cow::Borrowed(set),
            get_fcmo_bit: Cow::Borrowed(get_fcmo_bit),
            get_spmo_bit: Cow::Borrowed(get_spmo_bit),
        }
    }

    /// Create a new set of commands which uses heap allocated variants of types which could be
    /// constructed at compile time.
    pub const fn dynamic(set: String, get_fcmo_bit: String, get_spmo_bit: String) -> Self {
        Self {
            set: Cow::Owned(set),
            get_fcmo_bit: Cow::Owned(get_fcmo_bit),
            get_spmo_bit: Cow::Owned(get_spmo_bit),
        }
    }

    /// Create a new set of commands. Although more flexible than both [`Self::static`] and
    /// [`Self::dynamic`], you can only use this function at runtime.
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

/// System performance parameters which are passed as arguments to `acpi_call`.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceParameters {
    /// Parameter which is used to set the current system performance to intelligent cooling.
    pub intelligent_cooling: u32,

    /// Parameter which is used to set the current system performance to extreme performance.
    pub extreme_performance: u32,

    /// Parameter which is used to set the current system performance to battery saving.
    pub battery_saving: u32,
}

impl SystemPerformanceParameters {
    /// Shared parameters between Ideapad 15IIL05 and Ideapad AMD models.
    pub const SHARED: Self = Self {
        intelligent_cooling: 0x000FB001,
        extreme_performance: 0x0012B001,
        battery_saving: 0x0013B001,
    };

    /// Create a new set of system performance parameters.
    pub const fn new(
        intelligent_cooling: u32,
        extreme_performance: u32,
        battery_saving: u32,
    ) -> Self {
        Self {
            intelligent_cooling,
            extreme_performance,
            battery_saving,
        }
    }
}

/// System performance bits which are used to disambiguate between the different types of system
/// performance modes.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformanceBits {
    /// Intelligent cooling bit.
    pub intelligent_cooling: Bit,

    /// Extreme performance bit.
    pub extreme_performance: Bit,

    /// Battery saving bit.
    pub battery_saving: Bit,
}

impl SystemPerformanceBits {
    /// System performance bits which are shared between the Ideapad 15IIL05 and Ideapad AMD models.
    pub const SHARED: Self = Self {
        intelligent_cooling: Bit::same(0x0),
        extreme_performance: Bit::same(0x1),
        battery_saving: Bit::same(0x2),
    };

    /// Create a new set of system performance bits.
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

/// System performance configuration.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemPerformance {
    /// Commands for system performance.
    pub commands: SystemPerformanceCommands,

    /// Bits for system performance.
    pub bits: SystemPerformanceBits,

    /// Parameters for system performance.
    pub parameters: SystemPerformanceParameters,
}

impl SystemPerformance {
    /// Create a new system performance configuration.
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

/// Battery configuration for profile.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Battery {
    /// The command which is used to set both the battery conservation and rapid charge modes.
    pub set_command: Cow<'static, str>,

    /// Battery conservation configuration.
    pub conservation: SharedBatteryConfiguration,

    /// Rapid charge configuration.
    pub rapid_charge: SharedBatteryConfiguration,
}

impl Battery {
    /// Create a new battery configuration which uses stack allocated types which can be constructed
    /// at compile time.
    pub const fn r#static(
        set_command: &'static str,
        conservation: SharedBatteryConfiguration,
        rapid_charge: SharedBatteryConfiguration,
    ) -> Self {
        Self {
            set_command: Cow::Borrowed(set_command),
            conservation,
            rapid_charge,
        }
    }

    /// Create a new battery configuration which uses heap allocated types which can be constructed
    /// at compile time.
    pub const fn dynamic(
        set_command: String,
        conservation: SharedBatteryConfiguration,
        rapid_charge: SharedBatteryConfiguration,
    ) -> Self {
        Self {
            set_command: Cow::Owned(set_command),
            conservation,
            rapid_charge,
        }
    }

    /// Create a new battery configuration. Although more flexible than both [`Self::static`] and
    /// [`Self::dynamic`], this can only be used at runtime.
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

/// Parameters for [`SharedBatteryConfiguration`].
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SharedBatteryConfigurationParameters {
    /// Enable either battery conservation or rapid charge.
    pub enable: u32,

    /// Disable either battery conservation or rapid charge.
    pub disable: u32,
}

impl SharedBatteryConfigurationParameters {
    /// Shared battery conservation parameters which are shared between the Ideapad 15IIL05 and
    /// Ideapad AMD models.
    pub const CONSERVATION_SHARED: Self = Self {
        enable: 0x03,
        disable: 0x05,
    };

    /// Shared battery conservation parameters which are shared between the Ideapad 15IIL05 and
    /// Ideapad AMD models.
    pub const RAPID_CHARGE_SHARED: Self = Self {
        enable: 0x07,
        disable: 0x08,
    };

    /// Create new shared battery configuration parameters.
    pub const fn new(enable: u32, disable: u32) -> Self {
        Self { enable, disable }
    }
}

/// Battery configuration which is shared between battery conservation and rapid charge.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SharedBatteryConfiguration {
    /// The command to get either the battery conservation or rapid charge status.
    pub get_command: Cow<'static, str>,

    /// Parameters for battery conservation or rapid charge.
    pub parameters: SharedBatteryConfigurationParameters,
}

impl SharedBatteryConfiguration {
    /// Create a new battery configuration which uses stack allocated types which can be constructed
    /// at compile time.
    pub const fn r#static(
        get_command: &'static str,
        parameters: SharedBatteryConfigurationParameters,
    ) -> Self {
        Self {
            get_command: Cow::Borrowed(get_command),
            parameters,
        }
    }

    /// Create a new battery configuration which uses heap allocated types which can be constructed
    /// at compile time.
    pub const fn dynamic(
        get_command: String,
        parameters: SharedBatteryConfigurationParameters,
    ) -> Self {
        Self {
            get_command: Cow::Owned(get_command),
            parameters,
        }
    }

    /// Create a new battery configuration. Although more flexible than both [`Self::static`] and
    /// [`Self::dynamic`], this can only be used at runtime.
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

/// A configuration which allows this crate to be used in different Ideapad models.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Profile {
    /// The name of this profile.
    pub name: Cow<'static, str>,

    /// The product names which this profile supports.
    pub expected_product_names: Cow<'static, [Cow<'static, str>]>,

    /// System performance.
    pub system_performance: SystemPerformance,

    /// Battery.
    pub battery: Battery,
}

impl Profile {
    /// Default profile for the Ideapad 15IIL05 model. The only difference between this and the
    /// [`IDEAPAD_AMD`](Self::IDEAPAD_AMD) model is that instead of `LPC0`, it is `LPCB`.
    ///
    /// For example,
    ///
    /// The system performance set command for Ideapad 15IIL05 is:
    ///
    /// `\_SB.PCI0.LPCB.EC0.VPC0.DYTC`
    ///
    /// The system performance set command for Ideapad AMD is:
    ///
    /// `\_SB.PCI0.LPC0.EC0.VPC0.DYTC`
    ///
    /// The difference is highlighted here:
    ///
    /// ```text
    /// \_SB.PCI0.LPCB.EC0.VPC0.DYTC
    ///              ^
    /// \_SB.PCI0.LPC0.EC0.VPC0.DYTC
    ///              ^
    /// ```
    #[cfg(feature = "ideapad_15iil05")]
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
            ),
        ),
    );

    /// Default profile for the Ideapad AMD model. For the main differences between this and
    /// [`IDEAPAD_15IIL05`](Self::IDEAPAD_15IIL05), see it's respective documentation.
    #[cfg(feature = "ideapad_amd")]
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
            ),
        ),
    );

    /// Create a new profile which uses stack allocated variants of types which could be constructed
    /// at compile time.
    ///
    /// # Notes
    /// While you could provide `expected_product_names` an array of [`Cow`]s manually, you could
    /// also use the [`borrowed_cow_array`] macro to avoid boilerplate.
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

    /// Create a new profile which uses heap allocated variants of type which could be constructed
    /// at compile time.
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

    /// Create a new profile. Although more flexible than both [`Self::static`] and
    /// [`Self::dynamic`], it can only be constructed at runtime.
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

    /// Default search path for profiles.
    pub const SEARCH_PATH: &'static [Self] = &[
        #[cfg(feature = "ideapad_15iil05")]
        Self::IDEAPAD_15IIL05,
        #[cfg(feature = "ideapad_amd")]
        Self::IDEAPAD_AMD,
    ];

    /// Find the appropriate profile with the default search path.
    pub fn find() -> Result<Self> {
        Self::find_with_search_path(Self::SEARCH_PATH.iter().cloned())
    }

    /// Find the appropriate profile with the specified search path.
    ///
    /// # Errors
    /// If the system information couldn't be found, an [`Error::UnableToFindSystemInformation`] is
    /// returned.
    ///
    /// If this laptop's model's product name couldn't be found in the search path given, a
    /// [`Error::NoValidProfileInSearchPath`] is returned.
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
}

mod tests {}
