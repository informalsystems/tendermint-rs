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

impl ListSet<u128> {
    pub fn empty() -> Self {
        ListSet { list: List::Nil }
    }

    pub fn is_disjoint(&self, other: &ListSet<u128>) -> bool {
        is_equal(
            &self.list.contents().intersection(other.list.contents()),
            &Set::new(),
        )
    }

    pub fn contains(&self, t: &u128) -> bool {
        self.list.contents().contains(&t)
    }

    #[post(
      !self.contains(&t)
      && self.list.contents().is_subset(&old(&self).list.contents())
    )]
    pub fn remove(&mut self, t: &u128) {
        self.list.remove(t);
    }

    #[post(self.contains(&t))]
    pub fn insert(&mut self, t: u128) {
        self.list.insert(t);
    }

    pub fn first(&self) -> Option<u128> {
        match &self.list {
            List::Cons(t, _) => Some(*t),
            _ => None,
        }
    }
}

impl<V> ListMap<u128, V> {
    pub fn get(&self, key: &u128) -> Option<&V> {
        self.list.get(key)
    }

    pub fn contains(&self, key: &u128) -> bool {
        self.list.key_set().contains(&key)
    }

    pub fn contains_all(&self, keys: &ListSet<u128>) -> bool {
        is_equal(
            &self.list.key_set().intersection(keys.list.contents()),
            &keys.list.contents(),
        )
    }
}

fn is_equal<'a>(s1: &Set<&'a u128>, s2: &Set<&'a u128>) -> bool {
    s1.is_subset(s2) && s2.is_subset(s1)
}

impl List<u128> {
    #[measure(self)]
    pub fn contents(&self) -> Set<&u128> {
        match self {
            List::Nil => Set::new(),
            List::Cons(head, tail) => tail.contents().insert(head),
        }
    }

    #[post(
        !self.contents().contains(&t)
        && self.contents().is_subset(&old(&self).contents())
    )]
    fn remove(&mut self, t: &u128) {
        let list = std::mem::replace(self, List::Nil);
        let result = match list {
            List::Nil => List::Nil,
            List::Cons(head, mut tail) => {
                tail.remove(t);
                if head == *t {
                    *tail
                } else {
                    List::Cons(head, tail)
                }
            }
        };
        *self = result;
    }

    pub fn insert(&mut self, t: u128) {
        let list = std::mem::replace(self, List::Nil);
        *self = List::Cons(t, Box::new(list));
    }
}

impl<V> List<(u128, V)> {
    pub fn key_set(&self) -> Set<&u128> {
        match self {
            List::Nil => Set::new(),
            List::Cons(head, tail) => tail.key_set().insert(&head.0),
        }
    }

    pub fn get(&self, key: &u128) -> Option<&V> {
        match &self {
            List::Nil => None,
            List::Cons(head, _) if head.0 == *key => Some(&head.1),
            List::Cons(_, tail) => tail.get(key),
        }
    }
}
