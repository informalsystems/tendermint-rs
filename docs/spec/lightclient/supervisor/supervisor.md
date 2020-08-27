

TODO:

- initialization with genesis or light block
   - lightblock: lightstore is just initialized with it. nothing can
     be double-checked. Add explanation on that
   - genesis: download height 1, do detection with it
   
- at the end of the initialization the lightstore should contain a
  verified lightblock. 
  

- incorporate the structure of Stevan's Rust supervisor design
   - new versions of `verifytotarget` and `backwards` that take as
     input a single lightblock and return a fully verified lightstore
   - update tags to ".2"

 
check that all is addressed:

- https://github.com/informalsystems/tendermint-rs/issues/499
- https://github.com/informalsystems/tendermint-rs/pull/509
- https://github.com/tendermint/spec/issues/131
- https://github.com/informalsystems/tendermint-rs/issues/461


- put computation and submission if "minimal" PoF into a function that
  hides floating details
  
- links to verification and detection specs



# Light Client Sequential Supervisor



Roughly, it alternates two phases namely:
   - Light Client Verification. As a result, a header of the required
     height has been downloaded from and verified with the primary.
   - Light Client Fork Detections. As a result the header has been
     cross-checked with the secondaries. In case there is a fork we
     submit "proof of fork" and exit.
  


#### **[LC-FUNC-SUPERVISOR.1]:**

```go
func Sequential-Supervisor () (Error) {
    loop {
	    // get the next height
        nextHeight := input();
		
		// Verify
        result := NoResult;
        while result != ResultSuccess {
            lightStore,result := VerifyToTarget(primary, lightStore, nextHeight);
            if result == ResultFailure {				
				// pick new primary (promote a secondary to primary)
				/// and delete all lightblocks above
	            // LastTrusted (they have not been cross-checked)
	            Replace_Primary();
			}
        }
		
		// Cross-check
        PoFs := Forkdetector(lightStore, PoFs);
        if PoFs.Empty {
		    // no fork detected with secondaries, we trust the new
			// lightblock
            LightStore.Update(testedLB, StateTrusted);
        } 
        else {
		    // there is a fork, we submit the proofs and exit
            for i, p range PoFs {
                SubmitProofOfFork(p);
            } 
            return(ErrorFork);
        }
    }
}
```
**TODO:** finish conditions
- Implementation remark
- Expected precondition
    - *lightStore* initialized with trusted header
	- *PoFs* empty
- Expected postcondition
    - runs forever, or
	- is terminated by user and satisfies LightStore invariant, or **TODO**
	- has submitted proof of fork upon detecting a fork
- Error condition
    - none
----

# Semantics of the LightStore

Currently, a lightblock in the lightstore can be in one of the
following states:
- StateUnverified
- StateVerified
- StateFailed
- StateTrusted

The intuition is that `StateVerified` captures that the lightblock has
been verified with the primary, and `StateTrusted` is the state after
successful cross-checking with the secondaries.

Assuming there is **always one correct node among primary and
secondaries**, and there is no fork on the blockchain, lightblocks that
are in `StateTrusted` can be used by the user with the guarantee of
"finality". If a block in  `StateVerified` is used, it might be that
detection later finds a fork, and a roll-back might be needed.

**Remark:** The assumption of one correct node, does not render
verification useless. It is true that if the primary and the
secondaries return the same block we may trust it. However, if there
is a node that provides a different block, the light node still needs
verification to understand whether there is a fork, or whether the
different block is just bogus (without any support of some previous
validator set).

**Remark:** A light node may choose the full nodes it communicates
with (the light node and the full node might even belong to the same
stakeholder) so the assumption might be justified in some cases.

In the future, we will do the following changes
   - we assume that only from time to time, the light node is
     connected to a correct full node
   - this means for some limited time, the light node might have no
     means to defend against light client attacks
   - as a result we do not have finality
   - once the light node reconnects with a correct full node, it
     should detect the light client attack and submit evidence.

Under these assumptions, `StateTrusted` loses its meaning. As a
result, it should be removed from the API. We suggest that we replace
it with a flag "trusted" that can be used 
- internally for efficiency reasons (to maintain
  [LCD-INV-TRUSTED-AGREED.1] until a fork is detected)
- by light client based on the "one correct full node" assumption


----




