use once_cell::sync::OnceCell;
use crate::fallible_drop_strategy::FallibleDropStrategies;
use crate::{BatteryConservationController, Profile, RapidChargeController, SystemPerformanceController};

#[derive(Copy, Clone)]
pub struct Controllers<'ctx> {
    pub context: &'ctx Context,
}

impl<'ctx> Controllers<'ctx> {
    pub const fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    pub const fn battery_conservation(&self) -> BatteryConservationController<'ctx> {
        BatteryConservationController::new(self.context)
    }

    pub const fn rapid_charge(&self) -> RapidChargeController<'ctx> {
        RapidChargeController::new(self.context)
    }

    pub const fn system_performance(&self) -> SystemPerformanceController<'ctx> {
        SystemPerformanceController::new(self.context)
    }
}

pub struct Context {
    pub profile: Profile,
    fallible_drop_strategy: OnceCell<FallibleDropStrategies>,
}

impl Context {
    pub const fn new(profile: Profile) -> Self {
        Self {
            profile,
            fallible_drop_strategy: OnceCell::new(),
        }
    }

    pub fn with_fallible_drop_strategy(self, fallible_drop_strategy: FallibleDropStrategies) -> Self {
        let _ = self.fallible_drop_strategy.set(fallible_drop_strategy);
        self
    }

    pub fn fallible_drop_strategy(&self) -> &FallibleDropStrategies {
        self.fallible_drop_strategy.get_or_init(FallibleDropStrategies::default)
    }

    pub fn fallible_drop_strategy_mut(&mut self) -> &mut FallibleDropStrategies {
        if self.fallible_drop_strategy.get().is_none() {
            let _ = self.fallible_drop_strategy.set(FallibleDropStrategies::default());
        }

        self.fallible_drop_strategy.get_mut()
            .expect("expected fallible drop strategy to already be initialized after initializing it")
    }

    pub const fn controllers(&self) -> Controllers {
        Controllers::new(self)
    }
}