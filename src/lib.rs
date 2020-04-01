use std::fmt::{self, Display};
use std::marker::PhantomData;

mod macros;

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

    fn named(self, name: impl Into<String>) -> NamedPredicate<Self>
    where
        Self: Sized,
    {
        crate::named(self, name)
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

pub struct CurriedPredicate<A> {
    f: Box<dyn Fn(&A) -> bool>,
    a: A,
}

impl<A> CurriedPredicate<A> {
    pub fn new<F>(a: A, f: F) -> Self
    where
        F: Fn(&A) -> bool + 'static,
    {
        Self { f: Box::new(f), a }
    }
}

impl<A> Predicate for CurriedPredicate<A>
where
    A: Display,
{
    fn eval(&self) -> bool {
        (self.f)(&self.a)
    }
}

impl<A> Display for CurriedPredicate<A>
where
    A: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.a)
    }
}

pub struct TaggedPredicate<T> {
    pred: Box<dyn Predicate>,
    tag: PhantomData<T>,
}

impl<T> TaggedPredicate<T> {
    pub fn new(pred: impl Predicate + 'static) -> Self {
        Self {
            pred: Box::new(pred),
            tag: PhantomData,
        }
    }
}

impl<T> Predicate for TaggedPredicate<T> {
    fn eval(&self) -> bool {
        self.pred.eval()
    }
}

impl<T> Display for TaggedPredicate<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pred.fmt(f)
        // let tag_name = std::any::type_name::<T>();
        // write!(f, "{}@{}", tag_name, self.pred)
    }
}

pub struct NamedPredicate<P> {
    pred: P,
    name: String,
}

impl<P> NamedPredicate<P> {
    pub fn new(pred: P, name: impl Into<String>) -> Self {
        Self {
            pred,
            name: name.into(),
        }
    }
}

impl<P> Predicate for NamedPredicate<P>
where
    P: Predicate,
{
    fn eval(&self) -> bool {
        self.pred.eval()
    }
}

impl<P> Display for NamedPredicate<P>
where
    P: Predicate,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{}", self.name, self.pred)
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

pub fn tag<T>(pred: impl Predicate + 'static) -> TaggedPredicate<T> {
    TaggedPredicate::new(pred)
}

pub fn named<P>(pred: P, name: impl Into<String>) -> NamedPredicate<P> {
    NamedPredicate::new(pred, name)
}

pub fn pred<F, A>(f: F) -> impl Fn(A) -> CurriedPredicate<A>
where
    F: for<'r> Fn(&'r A) -> bool + Clone + 'static,
    A: 'static,
{
    move |a: A| CurriedPredicate::new(a, f.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use derive_more::Display;
    use quickcheck_macros::quickcheck;

    #[derive(Display)]
    #[display(fmt = "bar:{} > 0", bar)]
    struct Foo {
        bar: i32,
    }

    #[quickcheck]
    fn test_make_foo(bar: i32) -> bool {
        let make_foo = pred(|foo: &Foo| foo.bar > 0);
        let foo_pred = make_foo(Foo { bar });

        evals_to(foo_pred, bar > 0)
    }

    #[quickcheck]
    fn test_display_foo(bar: i32) -> bool {
        let make_foo = pred(|foo: &Foo| foo.bar > 0);

        let foo = Foo { bar };
        let foo_pred = make_foo(foo);

        foo_pred.to_string() == format!("bar:{} > 0", bar)
    }

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
