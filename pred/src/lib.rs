//! This library implements boolean-valued, fully saturated, boolean functions.
//! This allows combinatorial logic to be created, assembled and inspected at runtime.

use std::fmt::Display;
use std::marker::PhantomData;

#[cfg(feature = "inspect")]
pub mod inspect;

#[cfg(feature = "inspect")]
use crate::inspect::{Inspect, PredTree};

/// A fully saturated predicate which can be evaluated to a boolean value.
pub trait Predicate {
    /// Evaluates the predicate to a boolean value.
    fn eval(&self) -> bool;
}

/// Extension methods for predicates.
pub trait PredicateExt
where
    Self: Predicate,
{
    /// Build the conjunction of this predicate with `other`.
    fn and<P>(self, other: P) -> AndPredicate<Self, P>
    where
        Self: Sized,
    {
        AndPredicate {
            left: self,
            right: other,
        }
    }

    /// Build the disjunction of this predicate with `other`.
    fn or<P>(self, other: P) -> OrPredicate<Self, P>
    where
        Self: Sized,
    {
        OrPredicate {
            left: self,
            right: other,
        }
    }

    /// Build the negation of this predicate.
    fn not(self) -> NotPredicate<Self>
    where
        Self: Sized,
    {
        NotPredicate(self)
    }

    /// Build the implication of this predicate to `other`.
    fn implies<P>(self, other: P) -> ImpliesPredicate<Self, P>
    where
        Self: Sized,
    {
        ImpliesPredicate {
            assumption: self,
            conclusion: other,
        }
    }

    /// Convenience method to force evaluation of this predicate to `value`.
    ///
    /// ## TODO
    /// - Preserve underlying predicate
    fn constant(self, value: bool) -> ConstPredicate
    where
        Self: Sized,
    {
        ConstPredicate::new(value)
    }

    /// Attach a type-level tag to this predicate.
    #[cfg(not(feature = "inspect"))]
    fn tag<T>(self) -> TaggedPredicate<T>
    where
        Self: Sized + 'static,
    {
        crate::tag(self)
    }

    /// Attach a type-level tag to this predicate.
    #[cfg(feature = "inspect")]
    fn tag<T>(self) -> TaggedPredicate<T>
    where
        Self: Sized + Inspect + 'static,
    {
        crate::tag(self)
    }

    /// Provide a name for this predicate, which will be displayed when inspecting it.
    fn named(self, name: impl Into<String>) -> NamedPredicate<Self>
    where
        Self: Sized,
    {
        crate::named(self, name)
    }

    /// Box this predicate
    #[cfg(feature = "inspect")]
    fn boxed(self) -> BoxedPredicate
    where
        Self: Sized + Inspect + 'static,
    {
        crate::boxed(self)
    }

    /// Box this predicate
    #[cfg(not(feature = "inspect"))]
    fn boxed(self) -> BoxedPredicate
    where
        Self: Sized + 'static,
    {
        crate::boxed(self)
    }
}

impl<P: Predicate> PredicateExt for P {}

#[cfg(feature = "inspect")]
trait InspectablePredicate: Predicate + Inspect {}
#[cfg(feature = "inspect")]
impl<P> InspectablePredicate for P where P: Predicate + Inspect {}

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
            label: ("and".to_string(), self.eval()).into(),
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
            label: ("or".to_string(), self.eval()).into(),
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
            label: ("not".to_string(), self.eval()).into(),
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
            label: ("implies".to_string(), self.eval()).into(),
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

pub struct FnRefPredicate<F> {
    f: F,
}

impl<F> FnRefPredicate<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> Predicate for FnRefPredicate<F>
where
    F: Fn() -> bool,
{
    fn eval(&self) -> bool {
        (self.f)()
    }
}

#[cfg(feature = "inspect")]
impl<F> Inspect for FnRefPredicate<F>
where
    F: Fn() -> bool,
{
    fn inspect(&self) -> PredTree {
        PredTree::Leaf(("<function>".to_string(), self.eval()).into())
    }
}

pub struct FnPredicate<F> {
    f: F,
}

impl<F> FnPredicate<F> {
    pub fn new(f: F) -> Self {
        Self { f }
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
        PredTree::Leaf(("<function>".to_string(), self.eval()).into())
    }
}

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
            label: (self.name.clone(), self.eval()).into(),
            children: vec![self.pred.inspect()],
        }
    }
}

pub struct BoxedPredicate {
    #[cfg(feature = "inspect")]
    pred: Box<dyn InspectablePredicate>,
    #[cfg(not(feature = "inspect"))]
    pred: Box<dyn Predicate>,
}

impl BoxedPredicate {
    #[cfg(feature = "inspect")]
    pub fn new(pred: impl Predicate + Inspect + 'static) -> Self {
        Self {
            pred: Box::new(pred),
        }
    }

    #[cfg(not(feature = "inspect"))]
    pub fn new(pred: impl Predicate + 'static) -> Self {
        Self {
            pred: Box::new(pred),
        }
    }
}

impl Predicate for BoxedPredicate {
    fn eval(&self) -> bool {
        self.pred.eval()
    }
}

#[cfg(feature = "inspect")]
impl Inspect for BoxedPredicate {
    fn inspect(&self) -> PredTree {
        self.pred.inspect()
    }
}

/// Build a predicate which always evaluates to `value`
pub fn always(value: bool) -> ConstPredicate {
    ConstPredicate::new(value)
}

/// Build a predicate which always evaluates to the negation of `value`.
pub fn never(value: bool) -> ConstPredicate {
    always(!value)
}

/// Negate the given predicate.
pub fn not<P>(p: P) -> NotPredicate<P>
where
    P: Predicate,
{
    p.not()
}

/// Builds a predicate which evaluates to true when `left` is strictly smaller than `right`,
/// and to `false` otherwise.
pub fn less_than<T>(left: T, right: T) -> LessThanPredicate<T> {
    LessThanPredicate::new(left, right)
}

/// Builds a predicate which evaluates to the result of invoking the given closure.
pub fn from_fn<F>(f: F) -> FnPredicate<F>
where
    F: Fn() -> bool,
{
    FnPredicate::new(f)
}

/// Builds a predicate which evaluates to the result of invoking the given closure.
pub fn from_fn_ref<F>(f: F) -> FnRefPredicate<F>
where
    F: Fn() -> bool,
{
    FnRefPredicate::new(f)
}

/// Attach a type-level tag to this predicate.
#[cfg(feature = "inspect")]
pub fn tag<T>(pred: impl Predicate + Inspect + 'static) -> TaggedPredicate<T> {
    TaggedPredicate::new(pred)
}

/// Attach a type-level tag to this predicate.
#[cfg(not(feature = "inspect"))]
pub fn tag<T>(pred: impl Predicate + 'static) -> TaggedPredicate<T> {
    TaggedPredicate::new(pred)
}

/// Provide a name for the given predicate, which will be displayed when inspecting it.
pub fn named<P>(pred: P, name: impl Into<String>) -> NamedPredicate<P> {
    NamedPredicate::new(pred, name)
}

/// Box the given predicate
pub fn boxed(pred: impl Predicate + Inspect + 'static) -> BoxedPredicate {
    BoxedPredicate::new(pred)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use derive_more::Display;
    use quickcheck_macros::quickcheck;

    // #[derive(Display)]
    // #[display(fmt = "bar({} > 0)", self.bar[0])]
    struct Foo {
        bar: Vec<i32>,
    }

    #[quickcheck]
    fn test_from_fn(bar: i32) -> bool {
        let foo = Foo { bar: vec![bar] };
        let foo_pred = from_fn(|| foo.bar[0] > 0);

        evals_to(foo_pred, bar > 0)
    }

    #[quickcheck]
    fn test_from_fn_ref(bar: i32) -> bool {
        let foo = Foo { bar: vec![bar] };
        let p = make_foo_pred(&foo);
        let q = make_foo_pred(&foo);
        let pq = p.and(q); // .tag::<i32>();

        evals_to(pq, foo.bar[0] > 0)
    }

    fn make_foo_pred<'a>(foo: &'a Foo) -> impl InspectablePredicate + 'a {
        let foo_pred = from_fn(move || foo.bar[0] > 0);
        foo_pred
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
