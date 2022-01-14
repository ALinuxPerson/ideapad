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

use std::error::Error;
use std::{io, process};
use std::io::Write;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

static DROP_STRATEGY: Lazy<RwLock<Box<dyn DynFallibleDropStrategy>>> = Lazy::new(|| RwLock::new(Box::new(LogToWriterOnError::stderr())));

/// Marker trait which indicates that the implementing type is thread safe.
pub trait ThreadSafe: Send + Sync {}

impl<T: Send + Sync> ThreadSafe for T {}

/// This trait indicates that a structure can be used to handle errors that occur from drops.
pub trait FallibleDropStrategy: ThreadSafe {
    /// What to do on an error on a drop.
    fn on_error<E: Error>(&self, error: E);
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
struct LogToWriterOnError<W: Write + ThreadSafe> {
    writer: RwLock<W>,
}

impl LogToWriterOnError<io::Stderr> {
    /// Logs to standard error on error.
    pub fn stderr() -> Self {
        Self::new(io::stderr())
    }
}

impl<W: Write + ThreadSafe> LogToWriterOnError<W> {
    /// Logs to the specified writer on error.
    pub fn new(writer: W) -> Self {
        Self {
            writer: RwLock::new(writer),
        }
    }
}

impl<W: Write + ThreadSafe> FallibleDropStrategy for LogToWriterOnError<W> {
    fn on_error<E: Error>(&self, error: E) {
        let _ = writeln!(self.writer.write(), "error: {error}");
    }
}

struct PanicOnError;

impl FallibleDropStrategy for PanicOnError {
    fn on_error<E: Error>(&self, error: E) {
        panic!("{error}")
    }
}

pub(crate) fn on_error<E: Error>(error: E) {
    fn inner(error: &dyn Error) {
        DROP_STRATEGY.read().on_error(error)
    }

    inner(&error)
}

pub(crate) fn handle_error<E, F>(f: F)
where
    F: FnOnce() -> Result<(), E>,
    E: Error,
{
    if let Err(error) = f() {
        on_error(error)
    }
}

struct ExitOnError {
    exit_code: i32,
}

impl FallibleDropStrategy for ExitOnError {
    fn on_error<E: Error>(&self, _error: E) {
        process::exit(self.exit_code)
    }
}

struct DoNothingOnError;

impl FallibleDropStrategy for DoNothingOnError {
    fn on_error<E: Error>(&self, _error: E) {}
}

fn set<T>(strategy: T)
    where
        T: DynFallibleDropStrategy,
        T: 'static,
{
    fn inner(strategy: Box<dyn DynFallibleDropStrategy>) {
        *DROP_STRATEGY.write() = strategy
    }
    inner(Box::new(strategy))
}

/// Set the global [`FallibleDropStrategy`] to log to the specified writer on error.
pub fn log_to_writer_on_error<W>(writer: W)
    where
        W: Write + ThreadSafe,
        W: 'static,
{
    set(LogToWriterOnError::new(writer))
}

/// Set the global [`FallibleDropStrategy`] to log to standard output on error.
pub fn log_to_stdout_on_error() {
    log_to_writer_on_error(io::stdout())
}

/// Set the global [`FallibleDropStrategy`] to log to standard error on error.
pub fn log_to_stderr_on_error() {
    log_to_writer_on_error(io::stderr())
}

/// Set the global [`FallibleDropStrategy`] to panic on error.
pub fn panic_on_error() {
    set(PanicOnError)
}

/// Set the global [`FallibleDropStrategy`] to exit with the specified exit code on error.
pub fn exit_with_code_on_error(exit_code: i32) {
    set(ExitOnError { exit_code })
}

/// Set the global [`FallibleDropStrategy`] to exit on error.
pub fn exit_on_error() {
    exit_with_code_on_error(1)
}

/// Set the global [`FallibleDropStrategy`] to do nothing on error.
pub fn do_nothing_on_error() {
    set(DoNothingOnError)
}