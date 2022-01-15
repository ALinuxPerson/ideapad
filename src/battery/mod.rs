//! Shared contents between [`crate::battery_conservation`] and [`crate::rapid_charge`].
mod private {
    use crate::battery_conservation::BatteryConservationEnableGuard;
    use crate::rapid_charge::RapidChargeEnableGuard;
    use crate::{BatteryConservationController, RapidChargeController};

    pub trait BatteryEnableGuardSeal {}

    impl<'bc, 'ctx: 'bc> BatteryEnableGuardSeal for BatteryConservationEnableGuard<'bc, 'ctx> {}
    impl<'rc, 'ctx: 'rc> BatteryEnableGuardSeal for RapidChargeEnableGuard<'rc, 'ctx> {}

    pub trait BatteryControllerSeal {}

    impl<'ctx> BatteryControllerSeal for BatteryConservationController<'ctx> {}
    impl<'ctx> BatteryControllerSeal for RapidChargeController<'ctx> {}
}
use crate::{acpi_call, Handler};
use std::error::Error;

pub mod enable;

#[doc(hidden)]
#[allow(drop_bounds)]
pub trait BatteryEnableGuard<'ctrl, 'ctx: 'ctrl, C: BatteryController<'ctrl, 'ctx>>:
    Drop + Sized + private::BatteryEnableGuardSeal
{
    type Error: Error;

    fn new(controller: &'ctrl mut C, handler: Handler) -> Result<Self, Self::Error>;
}

#[doc(hidden)]
pub trait BatteryController<'this, 'ctx: 'this>: Sized + private::BatteryControllerSeal {
    type EnableGuard: BatteryEnableGuard<'this, 'ctx, Self>;
    type EnableError: Error + From<acpi_call::Error>;

    fn enable_ignore(&mut self) -> acpi_call::Result<()>;
    fn enable_error(&mut self) -> Result<(), Self::EnableError>;
    fn enable_switch(&mut self) -> acpi_call::Result<()>;
}
