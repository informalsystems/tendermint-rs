This release brings breaking changes, updating the `ExtendVote` request
data structure to the changes in the CometBFT 0.38.0 release.
The gRPC stack has been updated to prost 0.12 and tonic 0.10.
The RPC client for HTTP has been reimplemented using `reqwest`.
Support for Secp256k1 consensus keys has been added as an optional feature.
