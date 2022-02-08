use crate::battery_conservation::BatteryConservationEnableGuard;
use crate::rapid_charge::RapidChargeEnableGuard;
use crate::{BatteryConservationController, RapidChargeController};
use try_drop::prelude::*;

pub trait BatteryEnableGuardSeal {}

impl<'bc, 'ctx, D, DD> BatteryEnableGuardSeal for BatteryConservationEnableGuard<'bc, 'ctx, D, DD>
where
    'ctx: 'bc,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{}

impl<'rc, 'ctx: 'rc> BatteryEnableGuardSeal for RapidChargeEnableGuard<'rc, 'ctx> {}

pub trait BatteryDisableGuardSeal {}

pub trait BatteryControllerSeal {}

impl<'ctx, D, DD> BatteryControllerSeal for BatteryConservationController<'ctx, D, DD>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{}

impl<'ctx> BatteryControllerSeal for RapidChargeController<'ctx> {}
