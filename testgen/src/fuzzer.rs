/// A Fuzzer is anything that can produce an infinite random sequence of numbers.
/// 0 means no fuzzing, and any other number means fuzzing depending on the number.
pub trait Fuzzer<'a>: Iterator<Item = u64> {
}

pub struct FuzzIter<'a> {
    iter: &'a mut dyn Iterator<Item = u64>
}

impl<'a> FuzzIter<'a> {
    pub fn from<T: 'a + Iterator<Item=u64>>(iter: &'a mut T) -> Self {
        Self {
            iter
        }
    }
}

impl Iterator for FuzzIter<'_> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'a> Fuzzer<'a> for FuzzIter<'a> {}
