use tendermint_proto::abci::{RequestBeginBlock, RequestEndBlock, ResponseCommit, ResponseInfo};

#[derive(Debug, Default, Clone)]
pub struct ConsensusStateValidator {
    state: ConsensusState,
}

impl ConsensusStateValidator {
    pub fn on_info_response(&mut self, info_response: &ResponseInfo) {
        if self.state == ConsensusState::NoInfo {
            let block_height = info_response.last_block_height;

            if block_height == 0 {
                self.state = ConsensusState::NotInitialized;
            } else {
                self.state = ConsensusState::WaitingForBlock {
                    block_height: block_height + 1,
                    app_hash: info_response.last_block_app_hash.clone(),
                };
            }
        }
    }

    pub fn on_init_chain_request(&mut self) {
        if self.state != ConsensusState::NotInitialized {
            panic!("Received `InitChain` call when chain is already initialized");
        }

        self.state = ConsensusState::InitChain;
    }

    pub fn on_begin_block_request(&mut self, begin_block_request: &RequestBeginBlock) {
        let new_state = match self.state {
            ConsensusState::InitChain => {
                let header = begin_block_request
                    .header
                    .as_ref()
                    .expect("`BeginBlock` request does not contain a header");

                ConsensusState::ExecutingBlock {
                    block_height: header.height,
                    execution_state: BlockExecutionState::BeginBlock,
                }
            }
            ConsensusState::WaitingForBlock {
                ref block_height,
                ref app_hash,
            } => {
                let block_height = *block_height;

                let header = begin_block_request
                    .header
                    .as_ref()
                    .expect("`BeginBlock` request does not contain a header");

                if header.height != block_height {
                    panic!(
                        "Expected height {} in `BeginBlock` request. Got {}",
                        block_height, header.height
                    );
                }

                if &header.app_hash != app_hash {
                    panic!(
                        "Expected app hash {:?} in `BeginBlock`. Got {:?}",
                        app_hash, header.app_hash
                    );
                }

                ConsensusState::ExecutingBlock {
                    block_height,
                    execution_state: BlockExecutionState::BeginBlock,
                }
            }
            _ => panic!("`BeginBlock` cannot be called after {:?}", self.state),
        };

        self.state = new_state;
    }

    pub fn on_deliver_tx_request(&mut self) {
        match self.state {
            ConsensusState::ExecutingBlock {
                ref mut execution_state,
                ..
            } => execution_state.validate(BlockExecutionState::DeliverTx),
            _ => panic!("`DeliverTx` cannot be called after {:?}", self.state),
        }
    }

    pub fn on_end_block_request(&mut self, end_block_request: &RequestEndBlock) {
        match self.state {
            ConsensusState::ExecutingBlock {
                ref mut execution_state,
                ref block_height,
            } => {
                let block_height = *block_height;

                if block_height != end_block_request.height {
                    panic!(
                        "Expected `EndBlock` for height {}. But received for {}",
                        block_height, end_block_request.height
                    )
                }

                execution_state.validate(BlockExecutionState::EndBlock);
            }
            _ => panic!("`EndBlock` cannot be called after {:?}", self.state),
        }
    }

    pub fn on_commit_request(&mut self) {
        match self.state {
            ConsensusState::ExecutingBlock {
                ref mut execution_state,
                ..
            } => execution_state.validate(BlockExecutionState::Commit),
            _ => panic!("`Commit` cannot be called after {:?}", self.state),
        }
    }

    pub fn on_commit_response(&mut self, commit_response: &ResponseCommit) {
        let new_state = match self.state {
            ConsensusState::ExecutingBlock {
                execution_state: BlockExecutionState::Commit,
                block_height,
            } => ConsensusState::WaitingForBlock {
                block_height: block_height + 1,
                app_hash: commit_response.data.clone(),
            },
            _ => panic!("Received `CommitResponse` after {:?}", self.state),
        };

        self.state = new_state;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusState {
    NoInfo,
    NotInitialized,
    InitChain,
    WaitingForBlock {
        block_height: i64,
        app_hash: Vec<u8>,
    },
    ExecutingBlock {
        block_height: i64,
        execution_state: BlockExecutionState,
    },
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self::NoInfo
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockExecutionState {
    BeginBlock,
    DeliverTx,
    EndBlock,
    Commit,
}

impl BlockExecutionState {
    pub fn validate(&mut self, next: Self) {
        let is_valid = match (*self, next) {
            (Self::BeginBlock, Self::DeliverTx) => true,
            (Self::BeginBlock, Self::EndBlock) => true,
            (Self::DeliverTx, Self::DeliverTx) => true,
            (Self::DeliverTx, Self::EndBlock) => true,
            (Self::EndBlock, Self::Commit) => true,
            _ => false,
        };

        if is_valid {
            *self = next;
        } else {
            panic!("{:?} cannot be called after {:?}", next, self);
        }
    }
}
