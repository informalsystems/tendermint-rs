mod counter;
mod request_generator;

use tendermint_proto::abci::{response::Value as ResponseValue, Request, Response};

#[tokio::test]
async fn check_valid_abci_flow() {
    let server = counter::server();

    // First, tendermint calls `info` to get information about ABCI application
    let response = server.inner.process(request_generator::info()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Info(info_response) = response.value.unwrap() {
        assert_eq!(0, info_response.last_block_height);
        assert!(info_response.last_block_app_hash.is_empty());
    } else {
        panic!("Info request should generate info response");
    }

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let response = server.inner.process(request_generator::init_chain()).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let response = server
        .inner
        .process(request_generator::begin_block(1, Default::default()))
        .await;
    assert!(response.value.is_some());

    // Next, tendermint may call multiple `deliver_tx`
    let response = server.inner.process(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some());

    let response = server.inner.process(request_generator::deliver_tx(2)).await;
    assert!(response.value.is_some());

    // After all the transactions are delivered, tendermint will call `end_block`
    let response = server.inner.process(request_generator::end_block(1)).await;
    assert!(response.value.is_some());

    // Finally, tendermint will call `commit`
    let response = server.inner.process(request_generator::commit()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Commit(commit_response) = response.value.unwrap() {
        assert_eq!(2u64.to_be_bytes().to_vec(), commit_response.data);
    } else {
        panic!("Commit request should generate commit response");
    }

    // Next, tendermint will call `begin_block` with `block_height = 2`
    let response = server
        .inner
        .process(request_generator::begin_block(
            2,
            2u64.to_be_bytes().to_vec(),
        ))
        .await;
    assert!(response.value.is_some());

    // Next, tendermint may call multiple `deliver_tx`
    let response = server.inner.process(request_generator::deliver_tx(3)).await;
    assert!(response.value.is_some());

    let response = server.inner.process(request_generator::deliver_tx(4)).await;
    assert!(response.value.is_some());

    // After all the transactions are delivered, tendermint will call `end_block`
    let response = server.inner.process(request_generator::end_block(2)).await;
    assert!(response.value.is_some());

    // Finally, tendermint will call `commit`
    let response = server.inner.process(request_generator::commit()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Commit(commit_response) = response.value.unwrap() {
        assert_eq!(4u64.to_be_bytes().to_vec(), commit_response.data);
    } else {
        panic!("Commit request should generate commit response");
    }
}

#[tokio::test]
async fn check_valid_abci_flow_with_init_state() {
    let server = counter::server_with_state(4, 2);

    // First, tendermint calls `info` to get information about ABCI application
    let response = server.inner.process(request_generator::info()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Info(info_response) = response.value.unwrap() {
        assert_eq!(2, info_response.last_block_height);
        assert_eq!(
            4u64.to_be_bytes().to_vec(),
            info_response.last_block_app_hash
        );
    } else {
        panic!("Info request should generate info response");
    }

    // Because the `block_height` returned by `info` call is `2`, tendermint will next call
    // `begin_block` with `block_height = 3`
    let response = server
        .inner
        .process(request_generator::begin_block(
            3,
            4u64.to_be_bytes().to_vec(),
        ))
        .await;
    assert!(response.value.is_some());

    // Next, tendermint may call multiple `deliver_tx`
    let response = server.inner.process(request_generator::deliver_tx(5)).await;
    assert!(response.value.is_some());

    let response = server.inner.process(request_generator::deliver_tx(6)).await;
    assert!(response.value.is_some());

    // After all the transactions are delivered, tendermint will call `end_block`
    let response = server.inner.process(request_generator::end_block(3)).await;
    assert!(response.value.is_some());

    // Finally, tendermint will call `commit`
    let response = server.inner.process(request_generator::commit()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Commit(commit_response) = response.value.unwrap() {
        assert_eq!(6u64.to_be_bytes().to_vec(), commit_response.data);
    } else {
        panic!("Commit request should generate commit response");
    }
}

#[tokio::test]
async fn can_call_init_chain_after_startup() {
    let response = call_after_startup(request_generator::init_chain(), None).await;
    assert!(response.value.is_some());
}

#[tokio::test]
#[should_panic(expected = "`BeginBlock` cannot be called after NotInitialized")]
async fn cannot_call_begin_block_after_startup() {
    call_after_startup(request_generator::begin_block(0, Default::default()), None).await;
}

#[tokio::test]
#[should_panic(expected = "`DeliverTx` cannot be called after NotInitialized")]
async fn cannot_call_deliver_tx_after_startup() {
    call_after_startup(request_generator::deliver_tx(0), None).await;
}

#[tokio::test]
#[should_panic(expected = "`EndBlock` cannot be called after NotInitialized")]
async fn cannot_call_end_block_after_startup() {
    call_after_startup(request_generator::end_block(0), None).await;
}

#[tokio::test]
#[should_panic(expected = "`Commit` cannot be called after NotInitialized")]
async fn cannot_call_commit_after_startup() {
    call_after_startup(request_generator::commit(), None).await;
}

#[tokio::test]
#[should_panic(expected = "Received `InitChain` call when chain is already initialized")]
async fn cannot_call_init_chain_after_startup_with_state() {
    call_after_startup(request_generator::init_chain(), Some((1, 1))).await;
}

#[tokio::test]
#[should_panic(
    expected = "`DeliverTx` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
)]
async fn cannot_call_deliver_tx_after_startup_with_state() {
    call_after_startup(request_generator::deliver_tx(0), Some((1, 1))).await;
}

#[tokio::test]
#[should_panic(
    expected = "`EndBlock` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
)]
async fn cannot_call_end_block_after_startup_with_state() {
    call_after_startup(request_generator::end_block(0), Some((1, 1))).await;
}

#[tokio::test]
#[should_panic(
    expected = "`Commit` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
)]
async fn cannot_call_commit_after_startup_with_state() {
    call_after_startup(request_generator::commit(), Some((1, 1))).await;
}

#[tokio::test]
async fn can_call_begin_block_after_startup_with_state() {
    let response = call_after_startup(
        request_generator::begin_block(2, 1u64.to_be_bytes().to_vec()),
        Some((1, 1)),
    )
    .await;
    assert!(response.value.is_some())
}

#[tokio::test]
#[should_panic(expected = "Expected height 2 in `BeginBlock` request. Got 3")]
async fn cannot_call_begin_block_with_different_block_height_after_startup_with_state() {
    call_after_startup(
        request_generator::begin_block(3, 1u64.to_be_bytes().to_vec()),
        Some((1, 1)),
    )
    .await;
}

#[tokio::test]
#[should_panic(
    expected = "Expected app hash [0, 0, 0, 0, 0, 0, 0, 1] in `BeginBlock`. Got [0, 0, 0, 0, 0, 0, 0, 2]"
)]
async fn cannot_call_begin_block_with_different_app_hash_after_startup_with_state() {
    call_after_startup(
        request_generator::begin_block(2, 2u64.to_be_bytes().to_vec()),
        Some((1, 1)),
    )
    .await;
}

async fn call_after_startup(request: Request, state: Option<(u64, i64)>) -> Response {
    let (server, block_height, app_hash) = match state {
        None => (counter::server(), 0, Vec::new()),
        Some((counter, block_height)) => (
            counter::server_with_state(counter, block_height),
            block_height,
            counter.to_be_bytes().to_vec(),
        ),
    };

    // First, tendermint calls `info` to get information about ABCI application
    let response = server.inner.process(request_generator::info()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Info(info_response) = response.value.unwrap() {
        assert_eq!(block_height, info_response.last_block_height);
        assert_eq!(app_hash, info_response.last_block_app_hash);
    } else {
        panic!("Info request should generate info response");
    }

    // Send provided request
    server.inner.process(request).await
}

#[tokio::test]
#[should_panic(expected = "Received `InitChain` call when chain is already initialized")]
async fn cannot_call_init_chain_after_begin_block() {
    call_after_begin_block(request_generator::init_chain()).await;
}

#[tokio::test]
#[should_panic(
    expected = "`BeginBlock` cannot be called after ExecutingBlock { block_height: 1, execution_state: BeginBlock }"
)]
async fn cannot_call_begin_block_after_begin_block() {
    call_after_begin_block(request_generator::begin_block(2, Default::default())).await;
}

#[tokio::test]
#[should_panic(expected = "Commit cannot be called after BeginBlock")]
async fn cannot_call_commit_after_begin_block() {
    call_after_begin_block(request_generator::commit()).await;
}

#[tokio::test]
#[should_panic(expected = "Expected `EndBlock` for height 1. But received for 2")]
async fn cannot_call_end_block_with_different_block_height_after_begin_block() {
    call_after_begin_block(request_generator::end_block(2)).await;
}

#[tokio::test]
async fn can_call_deliver_tx_after_begin_block() {
    let response = call_after_begin_block(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some());
}

#[tokio::test]
async fn can_call_end_block_after_begin_block() {
    let response = call_after_begin_block(request_generator::end_block(1)).await;
    assert!(response.value.is_some());
}

async fn call_after_begin_block(request: Request) -> Response {
    let server = counter::server();

    // First, tendermint calls `info` to get information about ABCI application
    let response = server.inner.process(request_generator::info()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Info(info_response) = response.value.unwrap() {
        assert_eq!(0, info_response.last_block_height);
        assert!(info_response.last_block_app_hash.is_empty());
    } else {
        panic!("Info request should generate info response");
    }

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let response = server.inner.process(request_generator::init_chain()).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let response = server
        .inner
        .process(request_generator::begin_block(1, Default::default()))
        .await;
    assert!(response.value.is_some());

    // Send provided request
    server.inner.process(request).await
}

#[tokio::test]
#[should_panic(expected = "Received `InitChain` call when chain is already initialized")]
async fn cannot_call_init_chain_after_deliver_tx() {
    call_after_deliver_tx(request_generator::init_chain()).await;
}

#[tokio::test]
#[should_panic(
    expected = "`BeginBlock` cannot be called after ExecutingBlock { block_height: 1, execution_state: DeliverTx }"
)]
async fn cannot_call_begin_block_after_deliver_tx() {
    call_after_deliver_tx(request_generator::begin_block(
        2,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;
}

#[tokio::test]
#[should_panic(expected = "Commit cannot be called after DeliverTx")]
async fn cannot_call_commit_after_deliver_tx() {
    call_after_deliver_tx(request_generator::commit()).await;
}

#[tokio::test]
#[should_panic(expected = "Expected `EndBlock` for height 1. But received for 2")]
async fn cannot_call_end_block_with_different_height_after_deliver_tx() {
    call_after_deliver_tx(request_generator::end_block(2)).await;
}

#[tokio::test]
async fn can_call_deliver_tx_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some())
}

#[tokio::test]
async fn can_call_end_block_after_deliver_tx() {
    let response = call_after_deliver_tx(request_generator::end_block(1)).await;
    assert!(response.value.is_some())
}

async fn call_after_deliver_tx(request: Request) -> Response {
    let server = counter::server();

    // First, tendermint calls `info` to get information about ABCI application
    let response = server.inner.process(request_generator::info()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Info(info_response) = response.value.unwrap() {
        assert_eq!(0, info_response.last_block_height);
        assert!(info_response.last_block_app_hash.is_empty());
    } else {
        panic!("Info request should generate info response");
    }

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let response = server.inner.process(request_generator::init_chain()).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let response = server
        .inner
        .process(request_generator::begin_block(1, Default::default()))
        .await;
    assert!(response.value.is_some());

    // Next, tendermint will call `deliver_tx`
    let response = server.inner.process(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some());

    // Send provided request
    server.inner.process(request).await
}

#[tokio::test]
#[should_panic(expected = "Received `InitChain` call when chain is already initialized")]
async fn cannot_call_init_chain_after_end_block() {
    call_after_end_block(request_generator::init_chain()).await;
}

#[tokio::test]
#[should_panic(
    expected = "`BeginBlock` cannot be called after ExecutingBlock { block_height: 1, execution_state: EndBlock }"
)]
async fn cannot_call_begin_block_after_end_block() {
    call_after_end_block(request_generator::begin_block(
        2,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;
}

#[tokio::test]
#[should_panic(expected = "DeliverTx cannot be called after EndBlock")]
async fn cannot_call_deliver_tx_after_end_block() {
    call_after_end_block(request_generator::deliver_tx(2)).await;
}

#[tokio::test]
#[should_panic(expected = "EndBlock cannot be called after EndBlock")]
async fn cannot_call_end_block_after_end_block() {
    let respone = call_after_end_block(request_generator::end_block(1)).await;
    assert!(respone.value.is_some())
}

#[tokio::test]
async fn can_call_commit_after_end_block() {
    call_after_end_block(request_generator::commit()).await;
}

async fn call_after_end_block(request: Request) -> Response {
    let server = counter::server();

    // First, tendermint calls `info` to get information about ABCI application
    let response = server.inner.process(request_generator::info()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Info(info_response) = response.value.unwrap() {
        assert_eq!(0, info_response.last_block_height);
        assert!(info_response.last_block_app_hash.is_empty());
    } else {
        panic!("Info request should generate info response");
    }

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let response = server.inner.process(request_generator::init_chain()).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let response = server
        .inner
        .process(request_generator::begin_block(1, Default::default()))
        .await;
    assert!(response.value.is_some());

    // Next, tendermint will call `deliver_tx`
    let response = server.inner.process(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `end_block`
    let response = server.inner.process(request_generator::end_block(1)).await;
    assert!(response.value.is_some());

    // Send provided request
    server.inner.process(request).await
}

#[tokio::test]
#[should_panic(expected = "Received `InitChain` call when chain is already initialized")]
async fn cannot_call_init_chain_after_commit() {
    call_after_commit(request_generator::init_chain()).await;
}

#[tokio::test]
#[should_panic(expected = "Expected height 2 in `BeginBlock` request. Got 3")]
async fn cannot_call_begin_block_with_different_height_after_commit() {
    call_after_commit(request_generator::begin_block(
        3,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;
}

#[tokio::test]
#[should_panic(
    expected = "Expected app hash [0, 0, 0, 0, 0, 0, 0, 1] in `BeginBlock`. Got [0, 0, 0, 0, 0, 0, 0, 2]"
)]
async fn cannot_call_begin_block_with_different_app_hash_after_commit() {
    call_after_commit(request_generator::begin_block(
        2,
        2u64.to_be_bytes().to_vec(),
    ))
    .await;
}

#[tokio::test]
#[should_panic(
    expected = "`DeliverTx` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
)]
async fn cannot_call_deliver_tx_after_commit() {
    call_after_commit(request_generator::deliver_tx(2)).await;
}

#[tokio::test]
#[should_panic(
    expected = "`EndBlock` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
)]
async fn cannot_call_end_block_after_commit() {
    call_after_commit(request_generator::end_block(2)).await;
}

#[tokio::test]
#[should_panic(
    expected = "`Commit` cannot be called after WaitingForBlock { block_height: 2, app_hash: [0, 0, 0, 0, 0, 0, 0, 1] }"
)]
async fn cannot_call_commit_after_commit() {
    call_after_commit(request_generator::commit()).await;
}

#[tokio::test]
async fn can_call_begin_block_after_commit() {
    let response = call_after_commit(request_generator::begin_block(
        2,
        1u64.to_be_bytes().to_vec(),
    ))
    .await;
    assert!(response.value.is_some());
}

async fn call_after_commit(request: Request) -> Response {
    let server = counter::server();

    // First, tendermint calls `info` to get information about ABCI application
    let response = server.inner.process(request_generator::info()).await;
    assert!(response.value.is_some());
    if let ResponseValue::Info(info_response) = response.value.unwrap() {
        assert_eq!(0, info_response.last_block_height);
        assert!(info_response.last_block_app_hash.is_empty());
    } else {
        panic!("Info request should generate info response");
    }

    // Because the `block_height` returned by `info` call is `0`, tendermint will next call
    // `init_chain`
    let response = server.inner.process(request_generator::init_chain()).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `begin_block` with `block_height = 1`
    let response = server
        .inner
        .process(request_generator::begin_block(1, Default::default()))
        .await;
    assert!(response.value.is_some());

    // Next, tendermint will call `deliver_tx`
    let response = server.inner.process(request_generator::deliver_tx(1)).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `end_block`
    let response = server.inner.process(request_generator::end_block(1)).await;
    assert!(response.value.is_some());

    // Next, tendermint will call `commit`
    let response = server.inner.process(request_generator::commit()).await;
    assert!(response.value.is_some());

    // Send provided request
    server.inner.process(request).await
}
