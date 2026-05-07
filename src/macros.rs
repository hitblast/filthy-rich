//! Public, crate-only macros
//!

/// Generates a function `$name` which takes a `$param` and sets the
/// `$name` field of a struct to be a `None`-filtered `String`.
#[macro_export]
macro_rules! nf {
    ($name:ident, $doc:expr, $param:ident) => {
        #[must_use]
        #[doc = $doc]
        pub fn $name(mut self, $param: impl Into<String>) -> Self {
            let text = $param.into();
            self.$name = if !text.is_empty() { Some(text) } else { None };
            self
        }
    };
}

/// Generates a function `$name` which returns an `Option<&str>` by
/// dereferencing the `$name` field of a struct.
#[macro_export]
macro_rules! ds {
    ($name:ident, $doc:expr) => {
        #[must_use]
        #[doc = $doc]
        pub fn $name(&self) -> Option<&str> {
            self.$name.as_deref()
        }
    };
}
