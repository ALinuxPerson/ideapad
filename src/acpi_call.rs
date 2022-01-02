use std::{fs, io, iter};
use std::borrow::Cow;
use tap::Pipe;
use thiserror::Error;

const PATH: &str = "/proc/acpi/call";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("`acpi_call` kernel module not loaded")]
    KernelModuleNotLoaded { source: io::Error },

    #[error("unknown or unsupported value returned from `acpi_call`: '{value}'")]
    UnknownValue { value: String },

    #[error("unknown error returned from `acpi_call`: {message}")]
    UnknownError { message: String },

    #[error("method '{method}' not found in acpi table")]
    MethodNotFound { method: String },

    #[error("{error}")]
    Io {
        #[from]
        error: io::Error,
    },
}

impl Error {
    const AE_NOT_FOUND: &'static str = "AE_NOT_FOUND";

    pub fn maybe_method_not_found(message: String, method: String) -> Self {
        match message.as_str() {
            Self::AE_NOT_FOUND => Self::MethodNotFound { method },
            _ => Self::UnknownError { message },
        }
    }
}

pub(crate) enum Output {
    Valid(u32),
    Invalid(String),
}

pub(crate) fn acpi_call(
    command: String,
    parameters: impl IntoIterator<Item = u32>,
) -> Result<Output> {
    let command = iter::once(Cow::Borrowed(command.as_str()))
        .chain(
            parameters
                .into_iter()
                .map(|parameter| parameter.to_string())
                .map(Cow::Owned),
        )
        .collect::<Vec<_>>()
        .join(" ");

    if let Err(error) = fs::write(PATH, &command) {
        return if let io::ErrorKind::NotFound = error.kind() {
            Err(Error::KernelModuleNotLoaded { source: error })
        } else {
            Err(Error::Io { error })
        };
    }

    let output = fs::read_to_string(PATH)?.trim_end_matches('\0').to_string();

    if let Some(("Error", message)) = output.split_once(": ") {
        return Err(Error::maybe_method_not_found(message.to_string(), command));
    }

    if output.starts_with("0x") {
        Ok(output
            .trim_start_matches("0x")
            .pipe(|output| u32::from_str_radix(output, 16))
            .map(Output::Valid)
            .unwrap_or_else(|_| Output::Invalid(output)))
    } else {
        Ok(output
            .parse::<u32>()
            .map(Output::Valid)
            .unwrap_or_else(|_| Output::Invalid(output)))
    }
}

pub(crate) fn acpi_call_expect_valid(
    command: String,
    parameters: impl IntoIterator<Item = u32>,
) -> Result<u32> {
    match acpi_call(command, parameters) {
        Ok(Output::Valid(value)) => Ok(value),
        Ok(Output::Invalid(value)) => Err(Error::UnknownValue { value }),
        Err(error) => Err(error),
    }
}
