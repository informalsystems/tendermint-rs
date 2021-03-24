pub mod cmp {

    /// Stable version of `std::cmp::max_by_key`.
    pub fn max_by_key<A, B: Ord>(a: A, b: A, key: impl Fn(&A) -> B) -> A {
        if key(&a) > key(&b) {
            a
        } else {
            b
        }
    }

    /// Stable version of `std::cmp::min_by_key`.
    pub fn min_by_key<A, B: Ord>(a: A, b: A, key: impl Fn(&A) -> B) -> A {
        if key(&a) <= key(&b) {
            a
        } else {
            b
        }
    }
}

pub mod option {

    /// Choose between two optional values.
    ///
    /// If either value is `None`, the other one will be returned.
    /// If both values are `Some`, then the given function will pick one of the two values.
    pub fn select<A>(a: Option<A>, b: Option<A>, f: impl FnOnce(A, A) -> A) -> Option<A> {
        match (a, b) {
            (None, b) => b,
            (a, None) => a,
            (Some(a), Some(b)) => Some(f(a, b)),
        }
    }
}
