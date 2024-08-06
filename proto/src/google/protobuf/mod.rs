pub const PACKAGE: &str = "google.protobuf";

mod any;
pub use any::Any;

mod duration;
pub use duration::Duration;

mod timestamp;
pub use timestamp::Timestamp;

mod type_url;
