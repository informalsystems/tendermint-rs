use std::fmt::{self, Display};
use std::marker::PhantomData;

pub mod macros;

pub trait Predicate: Display {
    fn eval(&self) -> bool;
}

pub trait PredicateExt {
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

    fn implies<P>(self, other: P) -> ImpliesPredicate<Self, P>
    where
        Self: Sized,
    {
        ImpliesPredicate {
            assumption: self,
            conclusion: other,
        }
    }

    fn constant(self, value: bool) -> ConstPredicate
    where
        Self: Sized,
    {
        ConstPredicate::new(value)
    }

    fn tag<T>(self) -> TaggedPredicate<T>
    where
        Self: Sized + Predicate + 'static,
    {
        crate::tag(self)
    }
}

impl<P: Predicate> PredicateExt for P {}

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

impl Display for ConstPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

impl<P, Q> Display for AndPredicate<P, Q>
where
    P: Display,
    Q: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} && {})", self.left, self.right)
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

impl<P, Q> Display for OrPredicate<P, Q>
where
    P: Display,
    Q: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} || {})", self.left, self.right)
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

impl<P> Display for NotPredicate<P>
where
    P: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "!{}", self.0)
    }
}

pub struct ImpliesPredicate<P, Q> {
    assumption: P,
    conclusion: Q,
}

impl<P, Q> Predicate for ImpliesPredicate<P, Q>
where
    P: Predicate,
    Q: Predicate,
{
    fn eval(&self) -> bool {
        !self.assumption.eval() || self.conclusion.eval()
    }
}

impl<P, Q> Display for ImpliesPredicate<P, Q>
where
    P: Display,
    Q: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} ==> {})", self.assumption, self.conclusion)
    }
}

pub struct LessThanPredicate<T> {
    left: T,
    right: T,
}

impl<T> LessThanPredicate<T> {
    pub fn new(left: T, right: T) -> Self {
        Self { left, right }
    }
}

impl<T> Predicate for LessThanPredicate<T>
where
    T: PartialOrd,
    T: Display,
{
    fn eval(&self) -> bool {
        self.left.lt(&self.right)
    }
}

impl<T> Display for LessThanPredicate<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} < {})", self.left, self.right)
    }
}

pub struct FnPredicate<F> {
    f: F,
    descr: Option<String>,
}

impl<F> FnPredicate<F> {
    pub fn new(f: F) -> Self {
        Self { f, descr: None }
    }

    pub fn describe(self, descr: impl Into<String>) -> Self {
        Self {
            f: self.f,
            descr: Some(descr.into()),
        }
    }
}

impl<F> Predicate for FnPredicate<F>
where
    F: Fn() -> bool,
{
    fn eval(&self) -> bool {
        (self.f)()
    }
}

impl<F> Display for FnPredicate<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.descr {
            Some(ref descr) => write!(f, "<{}>", descr),
            None => write!(f, "<function>"),
        }
    }
}

pub struct TaggedPredicate<T> {
    predicate: Box<dyn Predicate>,
    tag: PhantomData<T>,
}

impl<T> Predicate for TaggedPredicate<T> {
    fn eval(&self) -> bool {
        self.predicate.eval()
    }
}

impl<T> Display for TaggedPredicate<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.predicate.fmt(f)
        // let tag_name = std::any::type_name::<T>();
        // write!(f, "[{}]@{}", self.predicate, tag_name)
    }
}

pub fn always(value: bool) -> ConstPredicate {
    ConstPredicate::new(value)
}

pub fn never(value: bool) -> ConstPredicate {
    always(!value)
}

pub fn not<P>(p: P) -> NotPredicate<P>
where
    P: Predicate,
{
    p.not()
}

pub fn less_than<T>(left: T, right: T) -> LessThanPredicate<T> {
    LessThanPredicate::new(left, right)
}

pub fn from_fn<F>(f: F) -> FnPredicate<F>
where
    F: Fn() -> bool,
{
    FnPredicate::new(f)
}

pub fn tag<T>(p: impl Predicate + 'static) -> TaggedPredicate<T> {
    TaggedPredicate {
        predicate: Box::new(p),
        tag: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn always_eval_to_value(value: bool) -> bool {
        let p = always(value);
        evals_to(p, value)
    }

    #[quickcheck]
    fn and_eval_to_conj(left: bool, right: bool) -> bool {
        let p = always(left);
        let q = always(right);
        evals_to(p.and(q), left && right)
    }

    #[quickcheck]
    fn or_eval_to_disj(left: bool, right: bool) -> bool {
        let p = always(left);
        let q = always(right);
        evals_to(p.or(q), left || right)
    }

    #[quickcheck]
    fn not_eval_to_neg(value: bool) -> bool {
        let p = always(value);
        evals_to(p.not(), !value)
    }

    #[quickcheck]
    fn implies_eval_to_implication(assumption: bool, conclusion: bool) -> bool {
        let p = always(assumption);
        let q = always(conclusion);
        evals_to(p.implies(q), !assumption || conclusion)
    }

    #[quickcheck]
    fn fn_eval_to_fn(value: bool) -> bool {
        let p = FnPredicate::new(|| value);
        evals_to(p, value)
    }

    fn evals_to(p: impl Predicate, result: bool) -> bool {
        p.eval() == result
    }
}
