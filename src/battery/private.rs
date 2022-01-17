use crate::battery_conservation::BatteryConservationEnableGuard;
use crate::rapid_charge::RapidChargeEnableGuard;
use crate::{BatteryConservationController, RapidChargeController};

pub trait BatteryEnableGuardSeal {}

impl<'bc, 'ctx: 'bc> BatteryEnableGuardSeal for BatteryConservationEnableGuard<'bc, 'ctx> {}

impl<'rc, 'ctx: 'rc> BatteryEnableGuardSeal for RapidChargeEnableGuard<'rc, 'ctx> {}

pub trait BatteryControllerSeal {}

impl<'ctx> BatteryControllerSeal for BatteryConservationController<'ctx> {}

impl<'ctx> BatteryControllerSeal for RapidChargeController<'ctx> {}
