#![allow(unused_variables, unreachable_code)]

use pred::*;

#[derive(Debug)]
pub struct Header;
#[derive(Debug)]
pub struct Commit;
#[derive(Debug)]
pub struct ValidatorSet;
#[derive(Debug)]
pub struct TrustThreshold {
    numerator: u64,
    denominator: u64,
}

pub struct IsEnoughPower;

predicate! { _self =>
    #[derive(Debug)]
    RealIsEnoughPower<>(
        signed_power: u64,
        total_power: u64,
        trust_threshold: TrustThreshold
    ) @ IsEnoughPower {
        signed_power * trust_threshold.denominator > total_power * trust_threshold.numerator
    } # "{:?}", _self
}

pub struct ValidCommit;

predicate! { _self =>
    #[derive(Debug)]
    RealValidCommit<>(
        header: Header,
        commit: Commit,
        trust_threshold: TrustThreshold
    ) @ ValidCommit {
        todo!()
    } # "{:?}", _self
}

predicate! { _self =>
    #[derive(Debug)]
    MockValidCommit<>(value: bool) @ ValidCommit {
        *value
    } # "{:?}", _self
}

pub struct Verify;
pub type VerifyPred = TaggedPredicate<Verify>;

pub fn real_verify_pred(header: Header, commit: Commit, trust_level: TrustThreshold) -> VerifyPred {
    let total_power = todo!();
    let signed_power = todo!();

    let is_enough_power = RealIsEnoughPower::pred(total_power, signed_power, trust_level);
    let valid_commit = RealValidCommit::pred(header, commit, trust_level);

    is_enough_power.and(valid_commit).tag()
}

pub fn mock_verify_pred(enough_power: bool, valid_commit: bool) -> VerifyPred {
    let is_enough_power = always(enough_power).named("is_enough_power");
    let valid_commit = always(valid_commit).named("is_valid_commit");

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
    // println!("\n{}", real_verify_pred());
    // let res = verify(real_verify_pred());
    // dbg!(res);

    let p = mock_verify_pred(true, false);

    println!("\n{}", p);
    let res = verify(p);
    dbg!(res);
}
