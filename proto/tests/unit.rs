#[test]
pub fn import_evidence_info() {
    use tendermint_proto::evidence::Info;
    let x = Info {
        committed: true,
        priority: 0,
        evidence: None,
    };
    assert_eq!(x.committed, true);
}
