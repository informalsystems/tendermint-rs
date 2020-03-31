pub trait Predicate {
    fn eval(&self) -> bool;

    fn and<P>(self, other: P) -> AndPredicate<Self, P>
    where
        Self: Sized,
    {
        AndPredicate {
            left: self,
            right: other,
        }
    }

    fn or<P>(self, other: P) -> OrPredicate<Self, P>
    where
        Self: Sized,
    {
        OrPredicate {
            left: self,
            right: other,
        }
    }

    fn not(self) -> NotPredicate<Self>
    where
        Self: Sized,
    {
        NotPredicate(self)
    }

    fn constant(self, value: bool) -> ConstPredicate
    where
        Self: Sized,
    {
        ConstPredicate::new(value)
    }
}

pub struct ConstPredicate(bool);

impl ConstPredicate {
    pub fn new(value: bool) -> Self {
        Self(value)
    }
}

impl Predicate for ConstPredicate {
    fn eval(&self) -> bool {
        self.0
    }
}

pub struct AndPredicate<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> Predicate for AndPredicate<P, Q>
where
    P: Predicate,
    Q: Predicate,
{
    fn eval(&self) -> bool {
        self.left.eval() && self.right.eval()
    }
}

pub struct OrPredicate<P, Q> {
    left: P,
    right: Q,
}

impl<P, Q> Predicate for OrPredicate<P, Q>
where
    P: Predicate,
    Q: Predicate,
{
    fn eval(&self) -> bool {
        self.left.eval() || self.right.eval()
    }
}

pub struct NotPredicate<P>(P);

impl<P> NotPredicate<P> {
    pub fn new(p: P) -> Self {
        Self(p)
    }
}

impl<P> Predicate for NotPredicate<P>
where
    P: Predicate,
{
    fn eval(&self) -> bool {
        !self.0.eval()
    }
}

pub struct FnPredicate<F>(F);

impl<F> FnPredicate<F> {
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<F> Predicate for FnPredicate<F>
where
    F: Fn() -> bool,
{
    fn eval(&self) -> bool {
        self.0()
    }
}

pub fn always(value: bool) -> ConstPredicate {
    ConstPredicate::new(value)
}

pub fn never(value: bool) -> ConstPredicate {
    always(!value)
}

pub fn not<P: Predicate>(p: P) -> NotPredicate<P> {
    p.not()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn const_eval_to_value(value: bool) -> bool {
        let p = ConstPredicate::new(value);
        is_eq(p, value)
    }

    #[quickcheck]
    fn and_eval_to_conj(left: bool, right: bool) -> bool {
        let p = ConstPredicate::new(left);
        let q = ConstPredicate::new(right);
        is_eq(p.and(q), left && right)
    }

    #[quickcheck]
    fn or_eval_to_disj(left: bool, right: bool) -> bool {
        let p = ConstPredicate::new(left);
        let q = ConstPredicate::new(right);
        is_eq(p.or(q), left || right)
    }

    #[quickcheck]
    fn not_eval_to_neg(value: bool) -> bool {
        let p = ConstPredicate::new(value);
        is_eq(p.not(), !value)
    }

    #[quickcheck]
    fn fn_eval_to_fn(value: bool) -> bool {
        let p = FnPredicate::new(|| value);
        is_eq(p, value)
    }

    fn is_eq(p: impl Predicate, result: bool) -> bool {
        p.eval() == result
    }

    // fn is_true(p: impl Predicate) -> bool {
    //     is_eq(p, true)
    // }

    // fn assert_true(p: impl Predicate) {
    //     assert!(is_true(p));
    // }
}
