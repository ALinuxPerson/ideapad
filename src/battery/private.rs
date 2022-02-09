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
{
}

impl<'rc, 'ctx, D, DD> BatteryEnableGuardSeal for RapidChargeEnableGuard<'rc, 'ctx, D, DD>
where
    'ctx: 'rc,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
}

pub trait BatteryDisableGuardSeal {}

pub trait BatteryControllerSeal {}

impl<'ctx, D, DD> BatteryControllerSeal for BatteryConservationController<'ctx, D, DD>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
}

impl<'ctx, D, DD> BatteryControllerSeal for RapidChargeController<'ctx, D, DD>
where
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
}
