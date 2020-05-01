/// Ensure a condition holds, returning an error if it doesn't (ala `assert`)
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $kind:expr) => {
        if !($cond) {
            return Err($kind.into());
        }
    };
}

/// Require that a precondition holds, panics if it doesn't.
#[macro_export]
macro_rules! precondition {
    ($cond:expr) => {
        debug_assert!($cond, "precondition failed: {}", stringify!($cond));
    };
}

/// Require that a precondition holds, panics if it doesn't.
#[macro_export]
macro_rules! postcondition {
    ($cond:expr) => {
        debug_assert!($cond, "postcondition failed: {}", stringify!($cond));
    };
}
