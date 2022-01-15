//! Shared contents between [`crate::battery_conservation`] and [`crate::rapid_charge`].
use std::error::Error;
use crate::{acpi_call, Handler};

pub mod enable;

pub trait BatteryEnableGuard<'ctrl, 'ctx: 'ctrl, C: BatteryController<'ctrl, 'ctx>>: Drop + Sized {
    type Error: Error;

    fn new(controller: &'ctrl mut C, handler: Handler) -> Result<Self, Self::Error>;
}

pub trait BatteryController<'this, 'ctx: 'this>: Sized {
    type EnableGuard: BatteryEnableGuard<'this, 'ctx, Self>;
    type EnableError: Error + From<acpi_call::Error>;

    fn enable_ignore(&mut self) -> acpi_call::Result<()>;
    fn enable_error(&mut self) -> Result<(), Self::EnableError>;
    fn enable_switch(&mut self) -> acpi_call::Result<()>;
}
