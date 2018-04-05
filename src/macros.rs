//! Various macros, mostly for error handling at least thus far

/// Create a new error (of a given enum variant) with a formatted message
macro_rules! err {
    ($variant:ident, $msg:expr) => {
        ::error::Error::$variant { description: $msg.to_owned() }
    };
    ($variant:ident, $fmt:expr, $($arg:tt)+) => {
        ::error::Error::$variant { description: format!($fmt, $($arg)+) }
    };
}

/// Create and return an error enum variant with a formatted message
// TODO: use all the macros
#[allow(unused_macros)]
macro_rules! fail {
    ($variant:ident, $msg:expr) => {
        return Err(err!($variant, $msg).into());
    };
    ($variant:ident, $fmt:expr, $($arg:tt)+) => {
        return Err(err!($variant, $fmt, $($arg)+).into());
    };
}

/// Assert a condition is true, returning an error type with a formatted message if not
// TODO: use all the macros
#[allow(unused_macros)]
macro_rules! ensure {
    ($condition: expr, $variant:ident, $msg:expr) => {
        if !($condition) {
            return Err(err!($variant, $msg).into());
        }
    };
    ($condition: expr, $variant:ident, $fmt:expr, $($arg:tt)+) => {
        if !($condition) {
            return Err(err!($variant, $fmt, $($arg)+).into());
        }
    };
}
