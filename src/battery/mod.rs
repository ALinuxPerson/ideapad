//! Shared contents between [`crate::battery_conservation`] and [`crate::rapid_charge`].
mod private;

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
