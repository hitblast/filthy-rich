//! Public, crate-only macros.
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
            self.$name = (!text.is_empty()).then_some(text);
            self
        }
    };
}

/// Generates a function which returns a borrowed `&str` for a given `String` field
/// named `$name` in a class.
#[macro_export]
macro_rules! str {
    ($name:ident, $doc:expr) => {
        #[must_use]
        #[doc = $doc]
        pub fn $name(&self) -> &str {
            &self.$name
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
