use light_spike::prelude::*;

pub fn main() {
    let store = TrustedStore::new();
    let (trusted_store_reader, trusted_store_writer) = store.split();

    let state = State {
        trusted_store_reader,
        trusted_store_writer,
    };

    let scheduler = RealScheduler;

    let predicates = light_spike::predicates::production::ProductionPredicates;
    let voting_power_calculator: Box<dyn VotingPowerCalculator> = todo(());
    let commit_validator: Box<dyn CommitValidator> = todo(());
    let header_hasher: Box<dyn HeaderHasher> = todo(());

    let verifier = RealVerifier::new(
        predicates,
        voting_power_calculator,
        commit_validator,
        header_hasher,
    );

    let header_hasher: Box<dyn HeaderHasher> = todo(());
    let fork_detector = RealForkDetector::new(header_hasher);

    let rpc_client = todo(());
    let io = RealIo::new(rpc_client);

    let demuxer = Demuxer::new(state, scheduler, verifier, fork_detector, io);
    todo(demuxer)
}

fn todo<A, B>(_: B) -> A {
    todo!()
}
