use tendermint_light_client_verifier::types::LightBlock;

#[derive(Clone, Debug)]
pub struct TraceTooShort {
    pub trace: Vec<LightBlock>,
}

#[derive(Clone, Debug)]
pub struct Trace(Vec<LightBlock>);

impl Trace {
    pub fn new(mut trace: Vec<LightBlock>) -> Result<Self, TraceTooShort> {
        if trace.len() < 2 {
            return Err(TraceTooShort { trace });
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

    pub fn iter(&self) -> std::slice::Iter<'_, LightBlock> {
        self.0.iter()
    }
}
