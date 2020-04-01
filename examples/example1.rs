#![allow(unused_variables, unreachable_code)]

use pred::*;

#[derive(Debug)]
pub struct Header;
#[derive(Debug)]
pub struct Commit;
#[derive(Debug)]
pub struct ValidatorSet;
#[derive(Copy, Clone, Debug)]
pub struct TrustThreshold {
    numerator: u64,
    denominator: u64,
}
impl std::fmt::Display for TrustThreshold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

pub struct IsEnoughPower;

predicate! { this =>
    #[derive(Debug)]
    RealIsEnoughPower<>(
        signed_power: u64,
        total_power: u64,
        trust_threshold: TrustThreshold
    ) @ IsEnoughPower {
        signed_power * trust_threshold.denominator > total_power * trust_threshold.numerator
    } # "real_is_enough_power(signed: {}, power: {}, trust_threshold: {})#{}",
        signed_power, total_power, trust_threshold, this.eval()
}

pub struct ValidCommit;

predicate! { this =>
    #[derive(Debug)]
    RealValidCommit<>(
        header: Header,
        commit: Commit,
        trust_threshold: TrustThreshold
    ) @ ValidCommit {
        true // TODO
    } # "real_valid_commit(header: {:?}, commit: {:?}, trust_threshold: {})#{}",
        header, commit, trust_threshold, this.eval()
}

pub struct Verify;
pub type VerifyPred = TaggedPredicate<Verify>;

pub fn real_verify_pred(header: Header, commit: Commit, trust_level: TrustThreshold) -> VerifyPred {
    let total_power = 42; // TODO
    let signed_power = 6;

    let is_enough_power = RealIsEnoughPower::pred(total_power, signed_power, trust_level);
    let valid_commit = RealValidCommit::pred(header, commit, trust_level);

    is_enough_power.and(valid_commit).tag()
}

pub fn mock_verify_pred(enough_power: bool, valid_commit: bool) -> VerifyPred {
    let is_enough_power = always(enough_power).named("mock_is_enough_power");
    let valid_commit = always(valid_commit).named("mock_is_valid_commit");

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
    let p = real_verify_pred(
        Header,
        Commit,
        TrustThreshold {
            numerator: 1,
            denominator: 3,
        },
    );

    println!("\n{}", p);
    let res = verify(p);
    dbg!(res);

    let p = mock_verify_pred(true, false);
    println!("\n{}", p);
    let res = verify(p);
    dbg!(res);
}
