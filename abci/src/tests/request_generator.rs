use tendermint_proto::{
    abci::{request::Value as RequestValue, *},
    types::*,
};

pub fn info() -> Request {
    let mut request = Request::default();
    request.value = Some(RequestValue::Info(RequestInfo::default()));
    request
}

pub fn init_chain() -> Request {
    let mut request = Request::default();
    request.value = Some(RequestValue::InitChain(Default::default()));
    request
}

pub fn begin_block(block_height: i64, app_hash: Vec<u8>) -> Request {
    let mut begin_block_request = RequestBeginBlock::default();

    let mut header = Header::default();
    header.height = block_height;
    header.app_hash = app_hash;

    begin_block_request.header = Some(header).into();

    let mut request = Request::default();
    request.value = Some(RequestValue::BeginBlock(begin_block_request));

    request
}

pub fn deliver_tx(counter: u64) -> Request {
    let mut deliver_tx_request = RequestDeliverTx::default();
    deliver_tx_request.tx = counter.to_be_bytes().to_vec();

    let mut request = Request::default();
    request.value = Some(RequestValue::DeliverTx(deliver_tx_request));

    request
}

pub fn end_block(block_height: i64) -> Request {
    let mut end_block_request = RequestEndBlock::default();
    end_block_request.height = block_height;

    let mut request = Request::default();
    request.value = Some(RequestValue::EndBlock(end_block_request));

    request
}

pub fn commit() -> Request {
    let mut request = Request::default();
    request.value = Some(RequestValue::Commit(RequestCommit::default()));
    request
}
