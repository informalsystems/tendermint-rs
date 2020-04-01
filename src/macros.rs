#[macro_export]
macro_rules! predicate {
    ( $name:ident ( $( $field:ident : $typ:ty ),* ) @ $tag:ident $eval:block # $($fmt:expr,)* ) => {
        paste::item! {
            pub type [<$name Pred>] = $crate::TaggedPredicate<$tag>;

            pub struct $name {
                $( $field: $typ, )*
            }

            impl $name {
                pub fn pred($( $field : $typ, )*) -> [<$name Pred>] {
                    let p = Self { $( $field, )* };
                    $crate::PredicateExt::tag(p)
                }
            }

            impl $crate::Predicate for $name {
                fn eval(&self) -> bool $eval
            }

            impl ::std::fmt::Display for $name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, $($fmt,)*)
                }
            }
        }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::Predicate;

    pub struct Tag;

    predicate! {
        Test(a: i32, b: bool) @ Tag {
            self.a > 0 && self.b
        } # "{} > 0 && {}", self.a, self.b,
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

// predicate! {
//     Test(a: u32, b: bool) @ Tag {
//         self.a > 0 && self.b
//     } # "{} > 0 && {}", self.a, self.b
// }
//
// pub type TestPred = TaggedPredicate<Tag>;
//
// #[derive(Debug)]
// pub struct Test {
//     a: u32,
//     b: bool
// }
//
// impl Test {
//     pub fn pred(a: u32, b: bool) -> TestPred {
//          let p = Self { a, b );
//          crate::PredicateExt::tag(p)
//     }
// }
//
// impl Predicate for Test {
//     fn eval(&self) -> bool {
//         self.a > 0 && self.b
//     }
// }
//
// impl fmt::Display for Test {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{} > 0 && {}", self.a, self.b)
//     }
// }
