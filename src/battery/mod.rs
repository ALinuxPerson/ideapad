//! Shared contents between [`crate::battery_conservation`] and [`crate::rapid_charge`].
mod private;

use crate::{acpi_call, Handler};
use std::error::Error;
use try_drop::PureTryDrop;

pub mod enable;

#[doc(hidden)]
#[allow(drop_bounds)]
pub trait BatteryEnableGuard<'ctrl, 'ctx: 'ctrl, C: BatteryController<'ctrl, 'ctx>>:
    Sized + private::BatteryEnableGuardSeal
{
    type Inner: PureTryDrop;

    fn new(controller: &'ctrl mut C, handler: Handler) -> Result<Self, C::Error>;
}

#[doc(hidden)]
#[allow(drop_bounds)]
pub trait BatteryDisableGuard<'ctrl, 'ctx: 'ctrl, C: BatteryController<'ctrl, 'ctx>>:
    Drop + Sized + private::BatteryDisableGuardSeal
{
    fn new(controller: &'ctrl mut C, handler: Handler) -> Result<Self, C::Error>;
}

#[doc(hidden)]
pub trait BatteryController<'this, 'ctx: 'this>: Sized + private::BatteryControllerSeal {
    type EnableGuard: BatteryEnableGuard<'this, 'ctx, Self>;
    type Error: Error + From<acpi_call::Error>;

    fn enable_ignore(&mut self) -> acpi_call::Result<()>;
    fn enable_error(&mut self) -> Result<(), Self::Error>;
    fn enable_switch(&mut self) -> acpi_call::Result<()>;
}
