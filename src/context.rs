//! Contains [`Context`], a structure which will be used by the majority of this crate.

use crate::fallible_drop_strategy::FallibleDropStrategies;
use crate::{profile, Profile};
use once_cell::sync::OnceCell;

#[cfg(feature = "battery_conservation")]
use crate::battery_conservation::BatteryConservationController;

#[cfg(feature = "rapid_charge")]
use crate::rapid_charge::RapidChargeController;

#[cfg(feature = "system_performance")]
use crate::system_performance::SystemPerformanceController;

/// Creates controllers.
#[derive(Copy, Clone)]
pub struct Controllers<'ctx> {
    /// A reference to the [`Context`].
    pub context: &'ctx Context,
}

impl<'ctx> Controllers<'ctx> {
    /// Creates a new [`Controllers`] instance.
    pub const fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Creates a new [`BatteryConservationController`] instance.
    #[cfg(feature = "battery_conservation")]
    pub const fn battery_conservation(&self) -> BatteryConservationController<'ctx> {
        BatteryConservationController::new(self.context)
    }

    /// Creates a new [`RapidChargeController`] instance.
    #[cfg(feature = "rapid_charge")]
    pub const fn rapid_charge(&self) -> RapidChargeController<'ctx> {
        RapidChargeController::new(self.context)
    }

    /// Creates a new [`SystemPerformanceController`] instance.
    #[cfg(feature = "system_performance")]
    pub const fn system_performance(&self) -> SystemPerformanceController<'ctx> {
        SystemPerformanceController::new(self.context)
    }
}

/// A context, which will be used by all controllers in this crate.
pub struct Context {
    /// The profile.
    pub profile: Profile,
    fallible_drop_strategy: OnceCell<FallibleDropStrategies>,
}

impl Context {
    /// Creates a new context.
    pub const fn new(profile: Profile) -> Self {
        Self {
            profile,
            fallible_drop_strategy: OnceCell::new(),
        }
    }

    /// Try and create a new context by trying to find a profile.
    pub fn try_default() -> profile::Result<Self> {
        Ok(Self::new(Profile::find()?))
    }

    /// Set the fallible drop strategy.
    ///
    /// # Notes
    /// If fallible drop strategy is already set, it won't be overwritten.
    pub fn with_fallible_drop_strategy(
        self,
        fallible_drop_strategy: FallibleDropStrategies,
    ) -> Self {
        let _ = self.fallible_drop_strategy.set(fallible_drop_strategy);
        self
    }

    /// Get a reference to the fallible drop strategy.
    pub fn fallible_drop_strategy(&self) -> &FallibleDropStrategies {
        self.fallible_drop_strategy
            .get_or_init(FallibleDropStrategies::default)
    }

    /// Get a mutable reference to the fallible drop strategy.
    pub fn fallible_drop_strategy_mut(&mut self) -> &mut FallibleDropStrategies {
        if self.fallible_drop_strategy.get().is_none() {
            let _ = self
                .fallible_drop_strategy
                .set(FallibleDropStrategies::default());
        }

        self.fallible_drop_strategy.get_mut().expect(
            "expected fallible drop strategy to already be initialized after initializing it",
        )
    }

    /// Create a controller creator.
    pub const fn controllers(&self) -> Controllers {
        Controllers::new(self)
    }
}
