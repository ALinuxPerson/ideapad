//! Macros which could be useful in this crate.

/// Create a statically borrowed clone on write array.
#[macro_export]
#[cfg(feature = "borrowed_cow_array")]
macro_rules! borrowed_cow_array {
    () => {
        &[]
    };
    ($($item:expr),+ $(,)?) => {
        &[$(::std::borrow::Cow::Borrowed($item)),+]
    }
}

/// Create a statically borrowed clone on write [`Vec`].
#[macro_export]
#[cfg(feature = "borrowed_cow_vec")]
macro_rules! borrowed_cow_vec {
    () => {
        vec![]
    };
    ($($item:expr),+ $(,)?) => {
        vec![$(::std::borrow::Cow::Borrowed($item)),+]
    }
}
