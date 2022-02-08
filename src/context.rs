//! Contains [`Context`], a structure which will be used by the majority of this crate.

use try_drop::prelude::*;
use crate::{profile, Profile};
use once_cell::sync::OnceCell;
use try_drop::{GlobalFallbackTryDropStrategyHandler, GlobalTryDropStrategyHandler};

#[cfg(feature = "battery_conservation")]
use crate::battery_conservation::BatteryConservationController;

#[cfg(feature = "rapid_charge")]
use crate::rapid_charge::RapidChargeController;

#[cfg(feature = "system_performance")]
use crate::system_performance::SystemPerformanceController;

/// Creates controllers.
#[derive(Copy, Clone)]
pub struct Controllers<'ctx, D = GlobalTryDropStrategyHandler, DD = GlobalFallbackTryDropStrategyHandler>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    /// A reference to the [`Context`].
    pub context: &'ctx Context<D, DD>,
}

impl<'ctx, D, DD> Controllers<'ctx, D, DD>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    /// Creates a new [`Controllers`] instance.
    pub fn new(context: &'ctx Context<D, DD>) -> Self {
        Self { context }
    }

    /// Creates a new [`BatteryConservationController`] instance.
    #[cfg(feature = "battery_conservation")]
    pub fn battery_conservation(&self) -> BatteryConservationController<'ctx, D, DD> {
        BatteryConservationController::new(self.context)
    }

    /// Creates a new [`RapidChargeController`] instance.
    #[cfg(feature = "rapid_charge")]
    pub fn rapid_charge(&self) -> RapidChargeController<'ctx, D, DD> {
        RapidChargeController::new(self.context)
    }

    /// Creates a new [`SystemPerformanceController`] instance.
    #[cfg(feature = "system_performance")]
    pub fn system_performance(&self) -> SystemPerformanceController<'ctx, D, DD> {
        SystemPerformanceController::new(self.context)
    }
}

/// A context, which will be used by all controllers in this crate.
pub struct Context<D = GlobalTryDropStrategyHandler, DD = GlobalFallbackTryDropStrategyHandler>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    /// The profile.
    pub profile: Profile,
    pub fallible_try_drop_strategy: D,
    pub fallback_try_drop_strategy: DD,
}

impl Context {
    /// Creates a new context.
    pub const fn new(profile: Profile) -> Self {
        Self {
            profile,
            fallible_try_drop_strategy: GlobalTryDropStrategyHandler,
            fallback_try_drop_strategy: GlobalFallbackTryDropStrategyHandler,
        }
    }

    /// Try and create a new context by trying to find a profile.
    pub fn try_default() -> profile::Result<Self> {
        Ok(Self::new(Profile::find()?))
    }
}

impl<D, DD> Context<D, DD>
    where
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    /// Creates a new context with the specified try drop strategies.
    pub fn new_with_strategies(profile: Profile, main: D, fallback: DD) -> Self {
        Self {
            profile,
            fallible_try_drop_strategy: main,
            fallback_try_drop_strategy: fallback,
        }
    }

    /// Try and create a new context by trying to find a profile.
    pub fn try_default_with_strategies(main: D, fallback: DD) -> profile::Result<Self> {
        Ok(Self::new_with_strategies(Profile::find()?, main, fallback))
    }

    /// Create a controller creator.
    pub fn controllers(&self) -> Controllers<D, DD> {
        Controllers::new(self)
    }
}