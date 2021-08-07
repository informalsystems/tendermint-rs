//! Small macros used internally.

/// Bail out of the current function with the given error kind.
#[macro_export]
macro_rules! bail {
    ($kind:expr) => {
        return Err($kind.into())
    };
}

/// Ensure a condition holds, returning an error if it doesn't (ala `assert`).
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $kind:expr) => {
        if !($cond) {
            return Err($kind.into());
        }
    };
}
