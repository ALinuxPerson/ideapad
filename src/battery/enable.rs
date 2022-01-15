use std::marker::PhantomData;
use crate::battery::{Controller, EnableGuard};
use crate::context::Context;
use crate::Handler;

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

pub struct EnableBuilder<'ctrl, 'ctx: 'ctrl, S: Stage, C: Controller<'ctrl, 'ctx>> {
    pub controller: &'ctrl mut C,
    stage: S,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctrl, 'ctx, C: Controller<'ctrl, 'ctx>> EnableBuilder<'ctrl, 'ctx, Begin, C> {
    pub fn new(controller: &'ctrl mut C) -> Self {
        Self {
            controller,
            stage: Begin { _priv: () },
            _marker: PhantomData,
        }
    }

    pub fn handler(self, handler: Handler) -> EnableBuilder<'ctrl, 'ctx, Call, C> {
        EnableBuilder {
            controller: self.controller,
            stage: Call { handler },
            _marker: PhantomData,
        }
    }

    pub fn ignore(self) -> EnableBuilder<'ctrl, 'ctx, Call, C> {
        self.handler(Handler::Ignore)
    }

    pub fn error(self) -> EnableBuilder<'ctrl, 'ctx, Call, C> {
        self.handler(Handler::Error)
    }

    pub fn switch(self) -> EnableBuilder<'ctrl, 'ctx, Call, C> {
        self.handler(Handler::Switch)
    }
}

impl<'ctrl, 'ctx: 'ctrl, C: Controller<'ctrl, 'ctx>> EnableBuilder<'ctrl, 'ctx, Call, C> {
    pub fn handler(&self) -> Handler {
        self.stage.handler
    }

    pub fn guard(self) -> Result<C::EnableGuard, <C::EnableGuard as EnableGuard<'ctrl, 'ctx, C>>::Error> {
        C::EnableGuard::new(self.controller, self.handler())
    }

    pub fn now(self) -> Result<(), C::EnableError> {
        match self.handler() {
            Handler::Ignore => self.controller.enable_ignore().map_err(Into::into),
            Handler::Error => self.controller.enable_error(),
            Handler::Switch => self.controller.enable_switch().map_err(Into::into),
        }
    }
}
