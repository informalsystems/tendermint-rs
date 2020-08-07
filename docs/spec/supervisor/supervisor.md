
```rust
/// Stores the LightBlocks which could not be confirmed to have originated from
/// the same chain as the predecesor light blocks.
type FetchedStack = Stack<LightBlock>;

/// Pair of light block and height of the light block which was the 
/// highest one for which we believe originated from the same chain
type ChainPair = (LightBlock, Height);

/// Traces the history of verify_to_target.
trait Trace {
    /// Returns the height of the current highest inserted
    fn current_height(&self) -> Height;

    /// Returns the current highest inserted light block.
    fn current(&self) -> &LightBlock;

    /// Insert a new highest light block. assert(self.current_height() < light_block.height())
    fn insert(&mut self, light_block: LightBlock) -> Result<(), Error>;

    /// Returns the list of all LightBlock which were inserted through 
    /// insert with associated previous current_height().
    fn chain(&self) -> LinkedList<ChainPair>;
}


/// Given a primary and a starting_light_block confirm that the light block of target_height received
/// from primary for target_height is from the same chain as the starting_light_block.
/// Solution for forward skipping verification.
pub fn verify_to_target(
    primary: PeerID,
    starting_light_block: LightBlock,
    target_height: Height) -> Result<Trace, (LightBlock, Reason)> {

    // Verified state always has one starting LightBlock
    let mut trace = Trace::from(starting_light_block);
    let mut fetch_stack = FetchedStack::empty();

    // insert the light block of target height, simplifies the loop
    // we can say that at each iteration there should be something on stack
    fetch_stack.push(fetch_light_block(primary, target_height));

    while trace.current_height() < targetHeight {
        // unverified state is never empty
        assert!(!fetch_stack.is_empty());
        // always try with previously received
        let lowest_unverified = fetch_stack.peek().unwrap().clone();
        
        // do the necessary checks
        let verdict = valid_and_verified(trace.current(), &lowest_unverified);

        match verdict {
            // validated LightBlock, improve the verified_state
            Valid => {
                fetch_stack.pop();
                trace.insert(lowest_unverified);
                continue;
            },
            // invalid block stop
            Invalid(reason) => {
                return Err((lowest_unverified, reason))
            },
            Untrusted => {
                // we need an intermediate LightBlock
                current_height = 
                    compute_height(trace.current_height(), lowest_unverified.height());
            }
        }
    }

    assert_eq!(trace.current_height(), target_height);
    assert!(fetch_stack.is_empty());

    return Ok(trace);
}

/// LightStore no longer has notions of Trusted, Verified, Unverified and Failed.
/// The LightBlocks which are stored are all assumed to have originated from the
/// same chain, irrespective of the peer from which we received it. The store
/// provides the invariant that there is a 1-1 mapping from height to a LightBlock.
trait LightStore {
    /// Current highest LightBlock which we believe to be from a chain 
    /// for which this store is associated with.
    fn highest(&self) -> LightBlock;

    /// Stores the list of ChainPair, possibly changing the highest LightBlock;
    /// Precondition is the existence of Light Block for which the chain can be linked
    /// with the already stored ChainPair. Failing if the chain can not be linked with
    /// existing history or if there is conflicting information.
    fn store_chain(&mut self, chain: LinkedList<ChainPair>) -> Result<(), Error>;
    
    /// Stores one ChainLink, failing if there is conflicting information in the store.
    /// (forks, etc.). Can be used when light blocks are received out of order.
    fn store_pair(&mut self, link: ChainLink) -> Result<(), Error>;

    /// Get verification chain of LightBlock ending at height high and starting starting from low.
    fn get_chain(&self, low: Height, high: Height) -> LinkedList<ChainPair>;

    /// In a case a fork is detected allows the light store to recover for a specified boundary
    /// height. Erases all light blocks from the store with height greater or equal to boundary.
    /// Returning the chain of light blocks which are removed. To be used if a fork is discovered
    /// after insertions of light blocks.
    fn recover(&mut self, boundary: Height) -> LinkedList<ChainPair>;

    /// Returns the firsts LightBlock above if it exists.
    fn above(&self, height: Height) -> Option<&LightBlock>;
    
    /// Returns the firsts LightBlock below if it exists.
    fn below(&self, height: Height) -> Option<&LightBlock>;
    
    /// Tries to get a certain height.
    fn get(&self, height: Height) -> Option<&LightBlock>;
}

/// Current understanding how a LightNode functions.
pub fn sequential_supervisor() -> Result<(), Forked> {
    loop {
	    // get the next height
        let target_height = input();
        // light_store contains all (LightBlock, previous), for this supervisor all have passed
        // fork detection and there will be no recoveries, etc.
		
		// Verify
        let mut result = Err(NoPeer);
        // try with primaries until you succeed or there are no more peers
        while result.is_err() {
            result = verify_to_target(get_primary(), trusted_store.heighest_trusted(), target_height);
            if result.is_err() {
                // if primary if faulty, replace it (no garbage, which should be cleaned)
                replace_primary()
            }
        }

        // safe verified state of the primary
        let result: Trace = result.unwrap();
        assert_eq!(target_height, result.current_height());
		
        // Cross-check
        let fork_result = fork_detector(witnesses(), light_store.heighest_trusted(), target_height);
        match fork_result {
            NoFork => {
                light_store.store_chain(result.verification_chain());
            }
            Fork(proof_of_fork) => {
                submit_evidence(proof_of_fork);
                return Err(Forked);
            }
        }
    }
    Ok(())
}
```
