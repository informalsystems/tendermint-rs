use std::future::Future;

use genawaiter::{rc::Gen, GeneratorState};

pub fn drain<I, O, E, F>(
    mut gen: Gen<I, O, F>,
    init: O,
    mut handler: impl FnMut(I) -> Result<O, E>,
) -> Result<F::Output, E>
where
    F: Future,
{
    let mut response = init;

    loop {
        match gen.resume_with(response) {
            GeneratorState::Yielded(request) => response = handler(request)?,
            GeneratorState::Complete(result) => return Ok(result),
        }
    }
}
