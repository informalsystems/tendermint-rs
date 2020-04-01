use pred::*;
use std::fmt;

pub struct IsEnoughPower;
pub type PredIsEnoughPower = TaggedPredicate<IsEnoughPower>;

#[derive(Debug)]
pub struct MockIsEnoughPower(bool);
impl MockIsEnoughPower {
    pub fn pred(value: bool) -> PredIsEnoughPower {
        Self(value).tag()
    }
}
impl Predicate for MockIsEnoughPower {
    fn eval(&self) -> bool {
        self.0
    }
}
impl fmt::Display for MockIsEnoughPower {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct RealIsEnoughPower;
impl RealIsEnoughPower {
    pub fn pred() -> PredIsEnoughPower {
        Self.tag()
    }
}
impl Predicate for RealIsEnoughPower {
    fn eval(&self) -> bool {
        true
    }
}
impl fmt::Display for RealIsEnoughPower {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RealIsEnoughPower(commit, header)")
    }
}

pub struct ValidCommit;
pub type PredValidCommit = TaggedPredicate<ValidCommit>;

pub struct RealValidCommit;
impl RealValidCommit {
    pub fn pred() -> PredValidCommit {
        Self.tag()
    }
}
impl Predicate for RealValidCommit {
    fn eval(&self) -> bool {
        true
    }
}
impl fmt::Display for RealValidCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RealValidCommit(commit, trust_level)")
    }
}

pub struct Verify;
pub type VerifyPred = TaggedPredicate<Verify>;

pub fn real_verify_pred() -> VerifyPred {
    let is_enough_power = RealIsEnoughPower::pred();
    let valid_commit = RealValidCommit::pred();

    is_enough_power.and(valid_commit).tag()
}

pub fn mock_verify_pred() -> VerifyPred {
    let is_enough_power = MockIsEnoughPower::pred(false);
    let valid_commit = RealValidCommit::pred();

    is_enough_power.and(valid_commit).tag()
}

pub fn verify(verify_pred: VerifyPred) -> Result<(), &'static str> {
    if verify_pred.eval() {
        Ok(())
    } else {
        Err("woops")
    }
}

#[allow(unused_must_use)]
fn main() {
    println!("\n{}", real_verify_pred());
    let res = verify(real_verify_pred());
    dbg!(res);

    println!("\n{}", mock_verify_pred());
    let res = verify(mock_verify_pred());
    dbg!(res);
}
