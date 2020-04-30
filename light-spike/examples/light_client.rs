use light_spike::components::scheduler;
use light_spike::prelude::*;

pub fn main() {
    let store = TrustedStore::new();
    let (trusted_store_reader, trusted_store_writer) = store.split();

    let state = State {
        trusted_store_reader,
        trusted_store_writer,
    };

    let predicates = light_spike::predicates::production::ProductionPredicates;
    let voting_power_calculator: Box<dyn VotingPowerCalculator> = todo(());
    let commit_validator: Box<dyn CommitValidator> = todo(());
    let header_hasher: Box<dyn HeaderHasher> = todo(());

    let scheduler = scheduler::handler(scheduler::process);
    let verifier = Verifier::new(
        predicates,
        voting_power_calculator,
        commit_validator,
        header_hasher,
    );

    let header_hasher: Box<dyn HeaderHasher> = todo(());
    let fork_detector = ForkDetector::new(header_hasher);

    let io = Io::new(todo(()));

    let demuxer = Demuxer::new(state, scheduler, verifier, fork_detector, io);
    todo(demuxer)
}

fn todo<A, B>(_: B) -> A {
    todo!()
}
