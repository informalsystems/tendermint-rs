/// Ensure a condition holds, returning an error if it doesn't (ala `assert`)
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $kind:expr) => {
        if !($cond) {
            return Err($kind.into());
        }
    };
}

