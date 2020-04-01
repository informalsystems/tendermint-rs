#[macro_export]
macro_rules! predicate {
    ( $name:ident ( $( $field:ident : $typ:ty ),* ) @ $tag:ident $eval:block # $($fmt:expr,)* ) => {
        paste::item! {
            pub type [<Pred $name>] = $crate::TaggedPredicate<$tag>;

            pub struct $name {
                $( $field: $typ, )*
            }

            impl $name {
                pub fn pred($( $field : $typ, )*) -> [<Pred $name>] {
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
    pub struct Tag;

    predicate! {
        Test(a: u32, b: bool) @ Tag {
            self.a > 0 && self.b
        } # "{} > 0 && {}", self.a, self.b,
    }
}

// predicate! {
//     Test(a: u32, b: bool) @ Tag {
//         self.a > 0 && self.b
//     } # "{} > 0 && {}", self.a, self.b
// }
//
// pub type PredTest = TaggedPredicate<Tag>;
//
// #[derive(Debug)]
// pub struct Test {
//     a: u32,
//     b: bool
// }
//
// impl Test {
//     pub fn pred(a: u32, b: bool) -> PredTest {
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
