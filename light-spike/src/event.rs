use std::any::Any;
use std::fmt::Debug;

#[macro_export]
macro_rules! impl_event {
    ($type:ty) => {
        #[::typetag::serde]
        impl $crate::event::Event for $type {
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn box_eq(&self, other: &dyn ::std::any::Any) -> bool {
                other.downcast_ref::<Self>().map_or(false, |a| self == a)
            }
        }
    };
}

#[typetag::serde(tag = "type")]
pub trait Event: Any + Debug {
    fn as_any(&self) -> &dyn Any;
    fn box_eq(&self, other: &dyn Any) -> bool;
}

pub type BoxedEvent = Box<dyn Event>;

impl PartialEq for BoxedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}

