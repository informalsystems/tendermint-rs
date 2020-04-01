#[macro_export]
macro_rules! predicate {
    ( $name:ident @ $tag:ident { $( $field:ident : $typ:ty, )* } => $eval:block # $($fmt:expr,)* ) => {
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
        Test @ Tag { a: u32, b: bool, } => {
            self.a > 0 && self.b
        } # "{} > 0 && {}", self.a, self.b,
    }
}

// predicate! {
//     MyPredicate @ SomeTag { a: u32, b: bool } => {
//         self.a > 0 && self.b
//     } # "{} > 0 && {}", self.a, self.b
// }
//
// pub type PredMyPredicate = TaggedPredicate<SomeTag>;
//
// #[derive(Debug)]
// pub struct MyPredicate {
//     a: u32,
//     b: bool
// }
//
// impl MyPredicate {
//     pub fn pred(a: u32, b: bool) -> PredMyPredicate {
//          (Self { a, b }).tag()
//     }
// }
//
// impl Predicate for MyPredicate {
//     fn eval(&self) -> bool {
//         self.a > 0 && self.b
//     }
// }
//
// impl fmt::Display for MyPredicate {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{} > 0 && {}", self.a, self.b)
//     }
// }
