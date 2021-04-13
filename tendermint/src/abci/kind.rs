/// A category of ABCI method.
///
/// ABCI methods are split into four categories. Tendermint opens one ABCI
/// connection for each category and refers to these categories as *connections*,
/// but nothing actually restricts an ABCI connection from calling methods in
/// multiple categories.
///
/// This enum breaks out the `Flush` method as a distinct category, since it is
/// used to control the execution of other methods.
pub enum MethodKind {
    /// A consensus method, driven by the consensus protocol and responsible for
    /// block execution.
    Consensus,
    /// A mempool method, used for validating new transactions before they're
    /// shared or included in a block.
    Mempool,
    /// A snapshot method, used for serving and restoring state snapshots.
    Snapshot,
    /// An info method, used for initialization and user queries.
    Info,
    /// The flush method requests that all pending method requests are fully executed.
    Flush,
}
