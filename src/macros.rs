//! Macros which could be useful in this crate.

/// Create a statically borrowed [`Cow`] array.
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

/// Create a statically borrowed [`Cow`] [`Vec`].
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
