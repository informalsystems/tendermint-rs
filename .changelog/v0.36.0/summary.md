This release brings substantial performance improvements to the voting power computation within the light client, improves the handling of misformed blocks (eg. with empty `last_commit` on non-first block) when decoding them from Protobuf or RPC responses, and adds missing `serde` derives on some Protobuf definitions.

This release also technically contains a breaking change in `tendermint-proto`, but this should not impact normal use of the library, as the `ToPrimitive` impl that was removed on `BlockIdFlag` trait did not provide any additional functionality.
