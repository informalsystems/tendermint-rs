#[macro_export]
macro_rules! unwrap {
    ($enum:path, $expr:expr) => {{
        if let $enum(item) = $expr {
            item
        } else {
            unreachable!()
        }
    }};
}
