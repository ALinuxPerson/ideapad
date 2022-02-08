//! Abstractions for enabling a battery mode.

use crate::battery::{BatteryController, BatteryEnableGuard};
use crate::context::Context;
use crate::Handler;
use std::marker::PhantomData;
use try_drop::prelude::*;

mod private {
    pub trait Sealed {}
}

/// A stage for the enable builder.
pub trait Stage: private::Sealed {}

/// The first stage.
///
/// This stage is where you specify the handler.
pub struct Begin {
    _priv: (),
}

impl Stage for Begin {}

impl private::Sealed for Begin {}

/// The second stage.
///
/// This stage is where you call the specified method you want, either create an enable guard or
/// enable immediately.
pub struct Call {
    handler: Handler,
}

impl Stage for Call {}

impl private::Sealed for Call {}

/// A builder for enabling a battery mode.
///
/// This is common between rapid charge and battery conservation.
pub struct EnableBuilder<'ctrl, 'ctx, S, C, D, DD>
where
    'ctx: 'ctrl,
    S: Stage,
    C: BatteryController<'ctrl, 'ctx>,
    D: FallibleTryDropStrategy,
    DD: FallbackTryDropStrategy,
{
    /// A reference to the controller, whatever that may be.
    pub controller: &'ctrl mut C,

    stage: S,
    _marker: PhantomData<&'ctx Context<D, DD>>,
}

impl<'ctrl, 'ctx, C, D, DD> EnableBuilder<'ctrl, 'ctx, Begin, C, D, DD>
    where
        'ctx: 'ctrl,
        C: BatteryController<'ctrl, 'ctx>,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    /// Start the process of enabling a battery mode.
    pub fn new(controller: &'ctrl mut C) -> Self {
        Self {
            controller,
            stage: Begin { _priv: () },
            _marker: PhantomData,
        }
    }

    /// Pick the handler, moving on to the next stage.
    pub fn handler(self, handler: Handler) -> EnableBuilder<'ctrl, 'ctx, Call, C, D, DD> {
        EnableBuilder {
            controller: self.controller,
            stage: Call { handler },
            _marker: PhantomData,
        }
    }

    /// Pick the ignore handler, moving on to the next stage.
    pub fn ignore(self) -> EnableBuilder<'ctrl, 'ctx, Call, C, D, DD> {
        self.handler(Handler::Ignore)
    }

    /// Pick the error handler, moving on to the next stage.
    pub fn error(self) -> EnableBuilder<'ctrl, 'ctx, Call, C, D, DD> {
        self.handler(Handler::Error)
    }

    /// Pick the switch handler, moving on to the next stage.
    pub fn switch(self) -> EnableBuilder<'ctrl, 'ctx, Call, C, D, DD> {
        self.handler(Handler::Switch)
    }
}

impl<'ctrl, 'ctx, C, D, DD> EnableBuilder<'ctrl, 'ctx, Call, C, D, DD>
    where
        'ctx: 'ctrl,
        C: BatteryController<'ctrl, 'ctx>,
        D: FallibleTryDropStrategy,
        DD: FallbackTryDropStrategy,
{
    /// Get the handler from the previous stage.
    pub fn handler(&self) -> Handler {
        self.stage.handler
    }

    /// Consume the builder, creating an enable guard from it.
    pub fn guard(
        self,
    ) -> Result<C::EnableGuard, C::Error> {
        C::EnableGuard::new(self.controller, self.handler())
    }

    /// Consume the builder, enabling the battery immediately with the handler that was specified
    /// from the previous stage.
    pub fn now(self) -> Result<(), C::Error> {
        match self.handler() {
            Handler::Ignore => self.controller.enable_ignore().map_err(Into::into),
            Handler::Error => self.controller.enable_error(),
            Handler::Switch => self.controller.enable_switch().map_err(Into::into),
        }
    }
}
