//! Fallible drop strategies.
//!
//! These are strategies that deal with how to go on when an error occurs in a [`Drop`]
//! implementation, usually in guard structures.
//!
//! The default fallible drop strategy is to log the error to the standard error stream.
//!
//! The following fallible drop strategies are available:
//!  * Logging to the standard error stream.
//!  * Logging to the standard output stream.
//!  * Logging to a specified writer.
//!  * Panic.
//!  * Do nothing/ignore the error.
//!
//! You may also implement your own fallible drop strategy and set it as the global drop strategy
//! via the [`FallibleDropStrategy`] trait and the [`set`] method.

use parking_lot::{Mutex, RwLock};
use std::error::Error;
use std::io::Write;
use std::ops::Deref;
use std::{io, process};

/// Marker trait which indicates that the implementing type is thread safe.
pub trait ThreadSafe: Send + Sync {}

impl<T: Send + Sync> ThreadSafe for T {}

/// Marker trait which indicates that the implementing types can be written to and is thread safe.
pub trait ThreadSafeWrite: ThreadSafe + Write {}

impl<T: ThreadSafe + Write> ThreadSafeWrite for T {}

/// Signifies that you can get an error from the implementing type.
pub trait CouldHandleError<E: Error> {
    fn get(self) -> Result<(), E>;
}

impl<E: Error> CouldHandleError<E> for Result<(), E> {
    fn get(self) -> Result<(), E> {
        self
    }
}

impl<F, E> CouldHandleError<E> for F
where
    F: FnOnce() -> Result<(), E>,
    E: Error,
{
    fn get(self) -> Result<(), E> {
        self()
    }
}

/// This trait indicates that a structure can be used to handle errors that occur from drops.
pub trait FallibleDropStrategy: ThreadSafe {
    /// What to do on an error on a drop.
    fn on_error<E: Error>(&self, error: E);
    fn handle_error<T, E>(&self, item: T)
    where
        T: CouldHandleError<E>,
        E: Error,
    {
        if let Err(error) = item.get() {
            self.on_error(error)
        }
    }
}

/// Dynamically dispatched version of [`FallibleDropStrategy`].
pub trait DynFallibleDropStrategy: ThreadSafe {
    /// Dynamically dispatched version of [`FallibleDropStrategy::on_error`].
    fn on_error(&self, error: &dyn Error);
}

impl<FDS: FallibleDropStrategy> DynFallibleDropStrategy for FDS {
    fn on_error(&self, error: &dyn Error) {
        self.on_error(error)
    }
}

/// A [`FallibleDropStrategy`] that logs to a specified writer on error.
pub struct LogToWriterOnError<W: ThreadSafeWrite> {
    writer: Mutex<W>,
}

impl<W: ThreadSafeWrite> LogToWriterOnError<W> {
    /// Logs to the specified writer on error.
    pub fn new(writer: W) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }
}

impl<W: ThreadSafeWrite> FallibleDropStrategy for LogToWriterOnError<W> {
    fn on_error<E: Error>(&self, error: E) {
        let _ = writeln!(self.writer.lock(), "error: {error}");
    }
}

pub struct PanicOnError;

impl FallibleDropStrategy for PanicOnError {
    fn on_error<E: Error>(&self, error: E) {
        panic!("{error}")
    }
}

pub struct ExitOnError {
    pub exit_code: i32,
}

impl FallibleDropStrategy for ExitOnError {
    fn on_error<E: Error>(&self, _error: E) {
        process::exit(self.exit_code)
    }
}

pub struct DoNothingOnError;

impl FallibleDropStrategy for DoNothingOnError {
    fn on_error<E: Error>(&self, _error: E) {}
}

pub enum DynWriter {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    Custom(Box<dyn ThreadSafeWrite>),
}

impl DynWriter {
    pub fn stdout() -> Self {
        Self::Stdout(io::stdout())
    }

    pub fn stderr() -> Self {
        Self::Stderr(io::stderr())
    }

    pub fn custom<W: ThreadSafeWrite + 'static>(writer: W) -> Self {
        Self::Custom(Box::new(writer))
    }
}

impl Write for DynWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(writer) => writer.write(buf),
            Self::Stderr(writer) => writer.write(buf),
            Self::Custom(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdout(writer) => writer.flush(),
            Self::Stderr(writer) => writer.flush(),
            Self::Custom(writer) => writer.flush(),
        }
    }
}

pub struct DynToGenericFallibleDropStrategyAdapter<'a>(pub &'a dyn DynFallibleDropStrategy);

impl<'a> FallibleDropStrategy for DynToGenericFallibleDropStrategyAdapter<'a> {
    fn on_error<E: Error>(&self, error: E) {
        DynFallibleDropStrategy::on_error(self.0, &error)
    }
}

pub enum FallibleDropStrategies {
    LogToWriterOnError(LogToWriterOnError<DynWriter>),
    PanicOnError(PanicOnError),
    ExitOnError(ExitOnError),
    DoNothingOnError(DoNothingOnError),
    Custom(Box<dyn DynFallibleDropStrategy>),
}

impl FallibleDropStrategies {
    pub const PANIC_ON_ERROR: Self = Self::PanicOnError(PanicOnError);
    pub const DO_NOTHING_ON_ERROR: Self = Self::DoNothingOnError(DoNothingOnError);

    pub fn log_to_writer_on_error<W: ThreadSafeWrite + 'static>(writer: W) -> Self {
        Self::LogToWriterOnError(LogToWriterOnError::new(DynWriter::custom(writer)))
    }

    pub fn log_to_stdout_on_error() -> Self {
        Self::LogToWriterOnError(LogToWriterOnError::new(DynWriter::stdout()))
    }

    pub fn log_to_stderr_on_error() -> Self {
        Self::LogToWriterOnError(LogToWriterOnError::new(DynWriter::stderr()))
    }

    pub const fn panic_on_error() -> Self {
        Self::PANIC_ON_ERROR
    }

    pub const fn exit_with_code_on_error(exit_code: i32) -> Self {
        Self::ExitOnError(ExitOnError { exit_code })
    }

    pub const fn exit_on_error() -> Self {
        Self::exit_with_code_on_error(1)
    }

    pub const fn do_nothing_on_error() -> Self {
        Self::DO_NOTHING_ON_ERROR
    }

    pub fn custom<T: DynFallibleDropStrategy + 'static>(fallible_drop_strategy: T) -> Self {
        Self::Custom(Box::new(fallible_drop_strategy))
    }
}

impl Default for FallibleDropStrategies {
    fn default() -> Self {
        FallibleDropStrategies::LogToWriterOnError(LogToWriterOnError::new(DynWriter::Stderr(
            io::stderr(),
        )))
    }
}

impl FallibleDropStrategy for FallibleDropStrategies {
    fn on_error<E: Error>(&self, error: E) {
        match self {
            FallibleDropStrategies::LogToWriterOnError(strategy) => {
                FallibleDropStrategy::on_error(strategy, error)
            }
            FallibleDropStrategies::PanicOnError(strategy) => {
                FallibleDropStrategy::on_error(strategy, error)
            }
            FallibleDropStrategies::ExitOnError(strategy) => {
                FallibleDropStrategy::on_error(strategy, error)
            }
            FallibleDropStrategies::DoNothingOnError(strategy) => {
                FallibleDropStrategy::on_error(strategy, error)
            }
            FallibleDropStrategies::Custom(strategy) => {
                // this *should* incur no overhead at runtime since this just stores a reference to
                // the dyn object
                let strategy = DynToGenericFallibleDropStrategyAdapter(strategy.deref());
                FallibleDropStrategy::on_error(&strategy, error)
            }
        }
    }
}
