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

use parking_lot::Mutex;
use std::error::Error;
use std::io::Write;
use std::ops::Deref;
use std::{io, process};

/// Marker trait which indicates that the implementing type is thread safe.
pub trait ThreadSafe: Send + Sync {}

impl<T: Send + Sync> ThreadSafe for T {}

/// Marker trait which indicates that the implementing types can be written to and is thread safe.
#[cfg(feature = "log_to_writer_on_error")]
pub trait ThreadSafeWrite: ThreadSafe + Write {}

#[cfg(feature = "log_to_writer_on_error")]
impl<T: ThreadSafe + Write> ThreadSafeWrite for T {}

/// Signifies that you can get an error from the implementing type.
pub trait CouldGetError {
    /// The error returned by [`Self::get`].
    type Error: Error;

    /// Gets the error.
    fn get(self) -> Result<(), Self::Error>;
}

impl<E: Error> CouldGetError for Result<(), E> {
    type Error = E;

    fn get(self) -> Result<(), Self::Error> {
        self
    }
}

impl<F, E> CouldGetError for F
where
    F: FnOnce() -> Result<(), E>,
    E: Error,
{
    type Error = E;

    fn get(self) -> Result<(), Self::Error> {
        self()
    }
}

/// This trait indicates that a structure can be used to handle errors that occur from drops.
pub trait FallibleDropStrategy: ThreadSafe {
    /// What to do on an error on a drop.
    fn on_error<E: Error>(&self, error: E);

    /// Handle an error on a drop.
    fn handle_error<T: CouldGetError>(&self, item: T) {
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
#[cfg(feature = "log_to_writer_on_error")]
pub struct LogToWriterOnError<W: ThreadSafeWrite> {
    writer: Mutex<W>,
}

#[cfg(feature = "log_to_writer_on_error")]
impl<W: ThreadSafeWrite> LogToWriterOnError<W> {
    /// Logs to the specified writer on error.
    pub fn new(writer: W) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }
}

#[cfg(feature = "log_to_writer_on_error")]
impl<W: ThreadSafeWrite> FallibleDropStrategy for LogToWriterOnError<W> {
    fn on_error<E: Error>(&self, error: E) {
        let _ = writeln!(self.writer.lock(), "error: {error}");
    }
}

/// A [`FallibleDropStrategy`] that panics on error.
#[cfg(feature = "panic_on_error")]
pub struct PanicOnError;

#[cfg(feature = "panic_on_error")]
impl FallibleDropStrategy for PanicOnError {
    fn on_error<E: Error>(&self, error: E) {
        panic!("{error}")
    }
}

/// A [`FallibleDropStrategy`] that exits with the specified `exit_code` on error.
#[cfg(feature = "exit_on_error")]
pub struct ExitOnError {
    /// The exit code to use.
    pub exit_code: i32,
}

#[cfg(feature = "exit_on_error")]
impl FallibleDropStrategy for ExitOnError {
    fn on_error<E: Error>(&self, _error: E) {
        process::exit(self.exit_code)
    }
}

/// A [`FallibleDropStrategy`] that ignores errors.
pub struct DoNothingOnError;

impl FallibleDropStrategy for DoNothingOnError {
    fn on_error<E: Error>(&self, _error: E) {}
}

/// A writer which attempts to use the most common variants if possible.
#[cfg(feature = "log_to_writer_on_error")]
pub enum DynWriter {
    /// A standard output writer.
    Stdout(io::Stdout),

    /// A standard error writer.
    Stderr(io::Stderr),

    /// A custom writer.
    Custom(Box<dyn ThreadSafeWrite>),
}

#[cfg(feature = "log_to_writer_on_error")]
impl DynWriter {
    /// Creates a new standard output writer.
    pub fn stdout() -> Self {
        Self::Stdout(io::stdout())
    }

    /// Creates a new standard error writer.
    pub fn stderr() -> Self {
        Self::Stderr(io::stderr())
    }

    /// Creates a new custom writer.
    pub fn custom<W: ThreadSafeWrite + 'static>(writer: W) -> Self {
        Self::Custom(Box::new(writer))
    }
}

#[cfg(feature = "log_to_writer_on_error")]
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

struct DynToGenericFallibleDropStrategyAdapter<'a>(pub &'a dyn DynFallibleDropStrategy);

impl<'a> FallibleDropStrategy for DynToGenericFallibleDropStrategyAdapter<'a> {
    fn on_error<E: Error>(&self, error: E) {
        DynFallibleDropStrategy::on_error(self.0, &error)
    }
}

/// A variety of drop strategies which offers the most common use cases, with a custom
/// [`FallibleDropStrategy`] in case you need to do something more specific.
pub enum FallibleDropStrategies {
    /// A [`FallibleDropStrategy`] that logs to a specified writer on error.
    #[cfg(feature = "log_to_writer_on_error")]
    LogToWriterOnError(LogToWriterOnError<DynWriter>),

    /// A [`FallibleDropStrategy`] that panics on error.
    #[cfg(feature = "panic_on_error")]
    PanicOnError(PanicOnError),

    /// A [`FallibleDropStrategy`] that exits with the specified `exit_code` on error.
    #[cfg(feature = "exit_on_error")]
    ExitOnError(ExitOnError),

    /// A [`FallibleDropStrategy`] that ignores errors.
    DoNothingOnError(DoNothingOnError),

    /// A custom [`FallibleDropStrategy`].
    Custom(Box<dyn DynFallibleDropStrategy>),
}

impl FallibleDropStrategies {
    /// A fallible drop strategy which panics on error.
    #[cfg(feature = "panic_on_error")]
    pub const PANIC_ON_ERROR: Self = Self::PanicOnError(PanicOnError);

    /// A fallible drop strategy which does nothing on error.
    pub const DO_NOTHING_ON_ERROR: Self = Self::DoNothingOnError(DoNothingOnError);

    /// A fallible drop strategy which logs to a specified writer on error.
    #[cfg(feature = "log_to_writer_on_error")]
    pub fn log_to_writer_on_error<W: ThreadSafeWrite + 'static>(writer: W) -> Self {
        Self::LogToWriterOnError(LogToWriterOnError::new(DynWriter::custom(writer)))
    }

    /// A fallible drop strategy which logs to standard output on error.
    #[cfg(feature = "log_to_writer_on_error")]
    pub fn log_to_stdout_on_error() -> Self {
        Self::LogToWriterOnError(LogToWriterOnError::new(DynWriter::stdout()))
    }

    /// A fallible drop strategy which logs to standard error on error.
    #[cfg(feature = "log_to_writer_on_error")]
    pub fn log_to_stderr_on_error() -> Self {
        Self::LogToWriterOnError(LogToWriterOnError::new(DynWriter::stderr()))
    }

    /// Returns [`Self::PANIC_ON_ERROR`].
    #[cfg(feature = "panic_on_error")]
    pub const fn panic_on_error() -> Self {
        Self::PANIC_ON_ERROR
    }

    /// A fallible drop strategy which exits with the specified `exit_code` on error.
    #[cfg(feature = "exit_on_error")]
    pub const fn exit_with_code_on_error(exit_code: i32) -> Self {
        Self::ExitOnError(ExitOnError { exit_code })
    }

    /// A fallible drop strategy which exits with code 1 on error.
    #[cfg(feature = "exit_on_error")]
    pub const fn exit_on_error() -> Self {
        Self::exit_with_code_on_error(1)
    }

    /// Returns [`Self::DO_NOTHING_ON_ERROR`].
    pub const fn do_nothing_on_error() -> Self {
        Self::DO_NOTHING_ON_ERROR
    }

    /// A custom fallible drop strategy.
    pub fn custom<T: DynFallibleDropStrategy + 'static>(fallible_drop_strategy: T) -> Self {
        Self::Custom(Box::new(fallible_drop_strategy))
    }
}

impl Default for FallibleDropStrategies {
    fn default() -> Self {
        #[cfg(feature = "log_to_writer_on_error")]
        {
            Self::log_to_stderr_on_error()
        }
        #[cfg(not(feature = "log_to_writer_on_error"))]
        {
            Self::do_nothing_on_error()
        }
    }
}

impl FallibleDropStrategy for FallibleDropStrategies {
    fn on_error<E: Error>(&self, error: E) {
        match self {
            #[cfg(feature = "log_to_writer_on_error")]
            FallibleDropStrategies::LogToWriterOnError(strategy) => {
                FallibleDropStrategy::on_error(strategy, error)
            }

            #[cfg(feature = "panic_on_error")]
            FallibleDropStrategies::PanicOnError(strategy) => {
                FallibleDropStrategy::on_error(strategy, error)
            }

            #[cfg(feature = "exit_on_error")]
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
