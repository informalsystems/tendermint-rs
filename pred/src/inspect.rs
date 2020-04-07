//! Provides datatypes and an `Inspect` trait for inspecting and pretty-printing predicates.

use colored::*;
use std::fmt::{self, Display};

#[cfg(feature = "inspect-dot")]
pub mod dot;

/// Packs together a string to be printed when inspecting a predicate,
/// as well as the result of its evaluation.
pub struct Label((String, bool));

impl From<(String, bool)> for Label {
    fn from(sb: (String, bool)) -> Self {
        Self(sb)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Label((label, result)) = self;
        if *result {
            write!(f, "{}", label.green())
        } else {
            write!(f, "{}", label.red())
        }
    }
}

/// A tree-like representation of a predicate and the result of its evaluation.
pub type PredTree = Tree<Label>;

/// This trait can be implemented by predicates which are to be inspected at runtime.
pub trait Inspect {
    /// Builds a tree-like representation of this predicate.
    ///
    /// See the `leaf` and `node` functions in this module for more
    /// information on how to build such trees.
    fn inspect(&self) -> PredTree;
}

impl<T: Inspect> Inspect for &T {
    fn inspect(&self) -> PredTree {
        (*self).inspect()
    }
}

/// Just a regular tree-like structure with named nodes.
pub enum Tree<A> {
    Leaf(A),
    Node { label: A, children: Vec<Tree<A>> },
}

/// Builds a leaf of the tree.
///
/// The provided `label` will be displayed (via its `Display` instance)
/// when pretty-printing the tree.
pub fn leaf<A>(label: A) -> Tree<A> {
    Tree::Leaf(label)
}

/// Builds a node of the tree.
///
/// The provided `label` will be displayed (via its `Display` instance)
/// when pretty-printing the tree.
pub fn node<A>(label: A, children: impl Into<Vec<Tree<A>>>) -> Tree<A> {
    Tree::Node {
        label,
        children: children.into(),
    }
}

impl<A> Tree<A> {
    pub fn label(&self) -> &A {
        match self {
            Self::Leaf(label) => label,
            Self::Node { label, .. } => label,
        }
    }

    pub fn is_leaf(&self) -> bool {
        if let Self::Leaf(_) = self {
            true
        } else {
            false
        }
    }
}

#[cfg(feature = "inspect-text")]
impl<A> Display for Tree<A>
where
    A: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn go<A: Display>(
            f: &mut fmt::Formatter<'_>,
            cur: &Tree<A>,
            level: usize,
            last: bool,
        ) -> fmt::Result {
            let sign = if level == 0 {
                ""
            } else if last && cur.is_leaf() {
                "└── "
            } else {
                "├── "
            };

            let padding = "│   ".repeat(if level > 0 { level - 1 } else { 0 });
            let prefix = format!("{}{}", padding, sign);

            match cur {
                Tree::Leaf(label) => {
                    writeln!(f, "{prefix}{}", label, prefix = prefix)?;
                }
                Tree::Node { label, children } => {
                    writeln!(f, "{prefix}{}", label, prefix = prefix)?;

                    for (idx, child) in children.iter().enumerate() {
                        go(f, child, level + 1, idx == children.len() - 1)?;
                    }
                }
            }

            Ok(())
        }

        go(f, self, 0, false)
    }
}

#[cfg(test)]
mod text_tests {
    use super::*;
    use fixtures::*;

    static FILES_SMALL_STR: &str = "pred
├── Cargo.lock
├── Cargo.toml
├── examples
│   └── example1.rs
├── src
│   ├── inspect.rs
│   ├── lib.rs
│   └── macros.rs
├── target
│   ├── debug
│   └── doc
";

    static FILES_BIG_STR: &str = "pred
├── Cargo.lock
├── Cargo.toml
├── examples
│   └── example1.rs
├── src
│   ├── inspect.rs
│   ├── lib.rs
│   ├── macros.rs
│   ├── bin
│   │   ├── foo.rs
│   │   └── bar.rs
├── target
│   ├── debug
│   ├── rls
│   │   ├── release
│   │   └── stuff.txt
│   └── doc
";

    #[test]
    fn inspect_files_small() {
        let formatted = format!("{}", files_small());
        assert_eq!(FILES_SMALL_STR, &formatted);
    }

    #[test]
    fn inspect_files_big() {
        let formatted = format!("{}", files_big());
        assert_eq!(FILES_BIG_STR, &formatted);
    }
}

#[cfg(test)]
mod fixtures {
    use super::*;

    pub fn files_small() -> Tree<String> {
        Tree::Node {
            label: "pred".to_string(),
            children: vec![
                Tree::Leaf("Cargo.lock".to_string()),
                Tree::Leaf("Cargo.toml".to_string()),
                Tree::Node {
                    label: "examples".to_string(),
                    children: vec![Tree::Leaf("example1.rs".to_string())],
                },
                Tree::Node {
                    label: "src".to_string(),
                    children: vec![
                        Tree::Leaf("inspect.rs".to_string()),
                        Tree::Leaf("lib.rs".to_string()),
                        Tree::Leaf("macros.rs".to_string()),
                    ],
                },
                Tree::Node {
                    label: "target".to_string(),
                    children: vec![
                        Tree::Leaf("debug".to_string()),
                        Tree::Leaf("doc".to_string()),
                    ],
                },
            ],
        }
    }

    pub fn files_big() -> Tree<String> {
        Tree::Node {
            label: "pred".to_string(),
            children: vec![
                Tree::Leaf("Cargo.lock".to_string()),
                Tree::Leaf("Cargo.toml".to_string()),
                Tree::Node {
                    label: "examples".to_string(),
                    children: vec![Tree::Leaf("example1.rs".to_string())],
                },
                Tree::Node {
                    label: "src".to_string(),
                    children: vec![
                        Tree::Leaf("inspect.rs".to_string()),
                        Tree::Leaf("lib.rs".to_string()),
                        Tree::Leaf("macros.rs".to_string()),
                        Tree::Node {
                            label: "bin".to_string(),
                            children: vec![
                                Tree::Leaf("foo.rs".to_string()),
                                Tree::Leaf("bar.rs".to_string()),
                            ],
                        },
                    ],
                },
                Tree::Node {
                    label: "target".to_string(),
                    children: vec![
                        Tree::Leaf("debug".to_string()),
                        Tree::Node {
                            label: "rls".to_string(),
                            children: vec![
                                Tree::Leaf("release".to_string()),
                                Tree::Leaf("stuff.txt".to_string()),
                            ],
                        },
                        Tree::Leaf("doc".to_string()),
                    ],
                },
            ],
        }
    }
}
