use tendermint_light_client::verifier::types::LightBlock;

use super::Error;

#[derive(Clone, Debug)]
pub struct Trace(Vec<LightBlock>);

impl Trace {
    pub fn new(mut trace: Vec<LightBlock>) -> Result<Self, Error> {
        if trace.len() < 2 {
            return Err(Error::trace_too_short(trace));
        }

        trace.sort_unstable_by_key(|lb| lb.height());

        Ok(Self(trace))
    }

    pub fn first(&self) -> &LightBlock {
        self.0.first().expect("trace is empty, which cannot happen")
    }

    pub fn last(&self) -> &LightBlock {
        self.0.last().expect("trace is empty, which cannot happen")
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, LightBlock> {
        self.0.iter()
    }

    pub fn into_vec(self) -> Vec<LightBlock> {
        self.0
    }
}

impl IntoIterator for Trace {
    type Item = LightBlock;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
