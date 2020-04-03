use std::fmt::Display;
use std::marker::PhantomData;

#[cfg(feature = "inspect")]
pub mod inspect;
#[cfg(feature = "inspect")]
pub use crate::inspect::Inspect;
#[cfg(feature = "inspect")]
use crate::inspect::PredTree;

pub trait Predicate {
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

    #[cfg(not(feature = "inspect"))]
    fn tag<T>(self) -> TaggedPredicate<T>
    where
        Self: Sized + Predicate + 'static,
    {
        crate::tag(self)
    }

    #[cfg(feature = "inspect")]
    fn tag<T>(self) -> TaggedPredicate<T>
    where
        Self: Sized + Predicate + Inspect + 'static,
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

#[cfg(feature = "inspect")]
impl Inspect for ConstPredicate {
    fn inspect(&self) -> PredTree {
        PredTree::Leaf((self.0.to_string(), self.eval()).into())
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

#[cfg(feature = "inspect")]
impl<P, Q> Inspect for AndPredicate<P, Q>
where
    P: Predicate + Inspect,
    Q: Predicate + Inspect,
{
    fn inspect(&self) -> PredTree {
        PredTree::Node {
            content: ("and".to_string(), self.eval()).into(),
            children: vec![self.left.inspect(), self.right.inspect()],
        }
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

#[cfg(feature = "inspect")]
impl<P, Q> Inspect for OrPredicate<P, Q>
where
    P: Predicate + Inspect,
    Q: Predicate + Inspect,
{
    fn inspect(&self) -> PredTree {
        PredTree::Node {
            content: ("or".to_string(), self.eval()).into(),
            children: vec![self.left.inspect(), self.right.inspect()],
        }
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

#[cfg(feature = "inspect")]
impl<P> Inspect for NotPredicate<P>
where
    P: Predicate + Inspect,
{
    fn inspect(&self) -> PredTree {
        PredTree::Node {
            content: ("not".to_string(), self.eval()).into(),
            children: vec![self.0.inspect()],
        }
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

#[cfg(feature = "inspect")]
impl<P, Q> Inspect for ImpliesPredicate<P, Q>
where
    P: Predicate + Inspect,
    Q: Predicate + Inspect,
{
    fn inspect(&self) -> PredTree {
        PredTree::Node {
            content: ("implies".to_string(), self.eval()).into(),
            children: vec![self.assumption.inspect(), self.conclusion.inspect()],
        }
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
    T: PartialOrd + Display,
{
    fn eval(&self) -> bool {
        self.left.lt(&self.right)
    }
}

#[cfg(feature = "inspect")]
impl<T> Inspect for LessThanPredicate<T>
where
    T: PartialOrd + Display,
{
    fn inspect(&self) -> PredTree {
        PredTree::Leaf((format!("{} < {}", self.left, self.right), self.eval()).into())
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

    pub fn descr(&self) -> &Option<String> {
        &self.descr
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

#[cfg(feature = "inspect")]
impl<F> Inspect for FnPredicate<F>
where
    F: Fn() -> bool,
{
    fn inspect(&self) -> PredTree {
        match self.descr() {
            Some(descr) => PredTree::Leaf((format!("<{}>", descr), self.eval()).into()),
            None => PredTree::Leaf(("<function>".to_string(), self.eval()).into()),
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

#[cfg(feature = "inspect")]
impl<A> Inspect for CurriedPredicate<A>
where
    A: Display,
{
    fn inspect(&self) -> PredTree {
        PredTree::Leaf((format!("{}", self.a), self.eval()).into())
    }
}

#[cfg(feature = "inspect")]
trait InspectablePredicate: Predicate + Inspect {}
#[cfg(feature = "inspect")]
impl<P> InspectablePredicate for P where P: Predicate + Inspect {}

pub struct TaggedPredicate<T> {
    #[cfg(feature = "inspect")]
    pred: Box<dyn InspectablePredicate>,
    #[cfg(not(feature = "inspect"))]
    pred: Box<dyn Predicate>,
    tag: PhantomData<T>,
}

impl<T> TaggedPredicate<T> {
    #[cfg(feature = "inspect")]
    pub fn new(pred: impl Predicate + Inspect + 'static) -> Self {
        Self {
            pred: Box::new(pred),
            tag: PhantomData,
        }
    }

    #[cfg(not(feature = "inspect"))]
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

#[cfg(feature = "inspect")]
impl<T> Inspect for TaggedPredicate<T> {
    fn inspect(&self) -> PredTree {
        self.pred.inspect()
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

    pub fn name(&self) -> &str {
        &self.name
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

#[cfg(feature = "inspect")]
impl<P> Inspect for NamedPredicate<P>
where
    P: Predicate + Inspect,
{
    fn inspect(&self) -> PredTree {
        PredTree::Node {
            content: (self.name.clone(), self.eval()).into(),
            children: vec![self.pred.inspect()],
        }
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

#[cfg(feature = "inspect")]
pub fn tag<T>(pred: impl Predicate + Inspect + 'static) -> TaggedPredicate<T> {
    TaggedPredicate::new(pred)
}

#[cfg(not(feature = "inspect"))]
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

pub fn tagged_pred<T, F, A>(f: F) -> impl Fn(A) -> TaggedPredicate<T>
where
    F: for<'r> Fn(&'r A) -> bool + Clone + 'static,
    A: Display + 'static,
{
    move |a: A| CurriedPredicate::new(a, f.clone()).tag()
}

#[cfg(test)]
mod tests {
    use super::*;
    use derive_more::Display;
    use quickcheck_macros::quickcheck;

    struct SomeTag;

    #[derive(Display)]
    #[display(fmt = "bar({} > 0)", bar)]
    struct Foo {
        bar: i32,
    }

    #[quickcheck]
    fn test_make_foo(bar: i32) -> bool {
        let make_foo = tagged_pred::<SomeTag, _, _>(|foo: &Foo| foo.bar > 0);
        let foo_pred = make_foo(Foo { bar });

        evals_to(foo_pred, bar > 0)
    }

    #[quickcheck]
    #[cfg(feature = "no-color")]
    fn test_display_foo_no_colors(bar: i32) -> bool {
        let make_foo = tagged_pred::<SomeTag, _, _>(|foo: &Foo| foo.bar > 0);

        let foo = Foo { bar };
        let foo_pred = make_foo(foo);

        foo_pred.inspect().to_string() == format!("bar({} > 0)\n", bar)
    }

    #[quickcheck]
    #[cfg(not(feature = "no-color"))]
    fn test_display_foo_colors(bar: i32) -> bool {
        use colored::Colorize;

        let make_foo = tagged_pred::<SomeTag, _, _>(|foo: &Foo| foo.bar > 0);

        let foo = Foo { bar };
        let foo_pred = make_foo(foo);

        let color = if bar > 0 { "green" } else { "red" };
        foo_pred.inspect().to_string() == format!("{}\n", format!("bar({} > 0)", bar).color(color))
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

