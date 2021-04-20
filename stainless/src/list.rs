use super::*;

#[derive(Clone)]
pub struct ListSet<T> {
    list: List<T>,
}

#[derive(Clone)]
pub struct ListMap<K, V> {
    list: List<(K, V)>,
}

#[derive(Clone)]
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}

impl<T: Eq + Hash + Clone> ListSet<T> {
    pub fn empty() -> Self {
        ListSet { list: List::Nil }
    }

    pub fn is_disjoint(&self, other: &ListSet<T>) -> bool {
        is_equal(
            &self.list.contents().intersection(other.list.contents()),
            &Set::empty(),
        )
    }

    pub fn contains(&self, t: &T) -> bool {
        self.list.contents().contains(&t)
    }

    pub fn remove(self, t: &T) -> Self {
        Self {
            list: self.list.remove(t),
        }
    }

    pub fn add(self, t: T) -> Self {
        Self {
            list: self.list.add(t),
        }
    }

    pub fn first(&self) -> Option<&T> {
        match &self.list {
            List::Cons(t, _) => Some(t),
            _ => None,
        }
    }
}

impl<K: Eq + Hash + Clone, V> ListMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        self.list.get(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.list.key_set().contains(&key)
    }

    pub fn contains_all(&self, keys: &ListSet<K>) -> bool {
        is_equal(
            &self.list.key_set().intersection(keys.list.contents()),
            &keys.list.contents(),
        )
    }
}

fn is_equal<T: Eq + Hash + Clone>(s1: &Set<T>, s2: &Set<T>) -> bool {
    s1.is_subset_of(s2) && s2.is_subset_of(s1)
}

impl<T: Eq + Hash + Clone> List<T> {
    #[measure(self)]
    pub fn contents(&self) -> Set<&T> {
        match self {
            List::Nil => Set::empty(),
            List::Cons(head, tail) => tail.contents().add(head),
        }
    }

    pub fn remove(self, t: &T) -> Self {
        match self {
            List::Nil => self,
            List::Cons(head, tail) if head == *t => *tail,
            List::Cons(head, tail) => List::Cons(head, Box::new(tail.remove(t))),
        }
    }

    pub fn add(self, t: T) -> Self {
        match self {
            List::Nil => List::Cons(t, Box::new(List::Nil)),
            _ => List::Cons(t, Box::new(self)),
        }
    }
}

impl<K: Eq + Hash + Clone, V> List<(K, V)> {
    pub fn key_set(&self) -> Set<&K> {
        match self {
            List::Nil => Set::empty(),
            List::Cons(head, tail) => tail.key_set().add(&head.0),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match &self {
            List::Nil => None,
            List::Cons(head, _) if head.0 == *key => Some(&head.1),
            List::Cons(_, tail) => tail.get(key),
        }
    }
}
