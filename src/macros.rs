/// A macro for generating `Predicate`s.
///
/// ## Example
///
/// When invoked this way:
///
/// ```rust,ignore
/// use pred::predicate;
///
/// pub struct Tag;
///
/// predicate! {
///     Test(a: u32, b: bool) {
///         self.a > 0 && self.b
///     }
///     @ Tag
///     # "{} > 0 && {}", self.a, self.b
/// }
/// ```
///
/// the macro expands to:
///
/// ```
/// pub struct Tag;
///
/// pub struct Test {
///     a: u32,
///     b: bool
/// }
///
/// impl Test {
///     pub fn new(a: u32, b: bool) -> Self {
///         Self { a, b }
///     }
///
///     pub fn pred(a: u32, b: bool) -> pred::TaggedPredicate<Tag> {
///          let __p = Self::new(a, b);
///          pred::PredicateExt::tag(__p)
///     }
/// }
///
/// impl pred::Predicate for Test {
///     fn eval(&self) -> bool {
///         let _self = &self;
///         let Test { a, b } = &self;
///         *a > 0 && *b
///     }
/// }
///
/// impl ::std::fmt::Display for Test {
///     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
///         let _self = &self;
///         let Test { a, b } = &self;
///         write!(f, "{} > 0 && {}", a, b)
///     }
/// }
/// ```
#[macro_export]
macro_rules! predicate {
    ( $self:ident =>
      $( #[ $attr:meta ] )*
      $name:ident
      < $( $param:ident : $cons:path ),* >
      ( $( $field:ident : $typ:ty ),* )
      $eval:block
      @ $tag:ident
      # $($fmt:expr),* ) => {

        #[allow(unused_code, unused_variables, unused_qualifications)]
        $( #[ $attr ] )*
        pub struct $name< $($param),* > {
            $( $field: $typ, )*
        }

        #[allow(unused_code, unused_variables, unused_qualifications)]
        impl < $($param),* > $name < $($param),* >
            where $( $param : $cons + 'static ),*
        {
            pub fn new($( $field : $typ, )*) -> Self {
                Self { $( $field, )* }
            }

            pub fn pred($( $field : $typ, )*) -> $crate::TaggedPredicate< $tag > {
                let __p = Self::new( $( $field, )* );
                $crate::PredicateExt::tag(__p)
            }
        }

        #[allow(unused_code, unused_variables, unused_qualifications)]
        impl < $($param),* > $crate::Predicate for $name < $($param),* >
            where $( $param : $cons ),*
        {
            fn eval(&self) -> bool {
                let $self = &self;
                let $name { $( $field, )* } = &self;
                $eval
            }
        }

        #[allow(unused_code, unused_variables, unused_qualifications)]
        impl < $($param),* > ::std::fmt::Display for $name < $($param),* >
            where $( $param : $cons ),*
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let $self = &self;
                let $name { $( $field, )* } = &self;
                write!(f, $($fmt,)*)
            }
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::Predicate;

    pub struct Tag;

    predicate! { _self =>
        #[derive(Debug)]
        Test<>(a: i32, b: bool) {
            *a > 0 && *b
        }
        @ Tag
        # "{} > 0 && {}", a, b
    }

    predicate! { _self =>
        Complex<P: Predicate, Q: Predicate>(p: P, q: Q) {
            p.eval() && !q.eval()
        }
        @ Tag
        # "{} && !{}", p, q
    }

    #[test]
    fn macro_pred_works_true() {
        let p = Test::pred(4, true);
        assert_eq!(p.eval(), true);
    }

    #[test]
    fn macro_pred_works_false1() {
        let p = Test::pred(-4, true);
        assert_eq!(p.eval(), false);
    }

    #[test]
    fn macro_pred_works_false2() {
        let p = Test::pred(4, false);
        assert_eq!(p.eval(), false);
    }

    #[test]
    fn macro_pred_format_works() {
        let p = Test::pred(4, true);
        assert_eq!(p.to_string(), "4 > 0 && true");
    }
}
