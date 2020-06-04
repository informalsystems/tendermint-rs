use std::fmt;

pub struct Callback<A> {
    inner: Box<dyn FnOnce(A) -> () + Send>,
}

impl<A> fmt::Debug for Callback<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callback").finish()
    }
}

impl<A> Callback<A> {
    pub fn new(inner: impl FnOnce(A) -> () + Send + 'static) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    pub fn call(self, result: A) -> () {
        (self.inner)(result);
    }
}
