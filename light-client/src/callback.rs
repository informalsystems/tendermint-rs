use std::fmt;

/// A boxed `FnOnce(A) -> () + Send`.
pub struct Callback<A> {
    inner: Box<dyn FnOnce(A) -> () + Send>,
}

impl<A> fmt::Debug for Callback<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callback").finish()
    }
}

impl<A> Callback<A> {
    /// Box the given closure in a `Callback`.
    pub fn new(inner: impl FnOnce(A) -> () + Send + 'static) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    /// Call the underlying closure on `result`.
    pub fn call(self, result: A) -> () {
        (self.inner)(result);
    }
}
