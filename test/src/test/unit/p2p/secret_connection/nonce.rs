use std::collections::HashMap;

use tendermint_p2p::secret_connection::{Nonce, NONCE_SIZE};

#[test]
fn test_incr_nonce() {
    // make sure we match the golang implementation
    let mut check_points: HashMap<i32, &[u8]> = HashMap::new();
    check_points.insert(0, &[0_u8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
    check_points.insert(1, &[0_u8, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
    check_points.insert(510, &[0_u8, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0]);
    check_points.insert(511, &[0_u8, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0]);
    check_points.insert(512, &[0_u8, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0]);
    check_points.insert(1023, &[0_u8, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0]);

    let mut nonce = Nonce::default();
    assert_eq!(nonce.to_bytes().len(), NONCE_SIZE);

    for i in 0..1024 {
        nonce.increment();
        if let Some(want) = check_points.get(&i) {
            let got = &nonce.to_bytes();
            assert_eq!(got, want);
        }
    }
}
#[test]
#[should_panic]
fn test_incr_nonce_overflow() {
    // other than in the golang implementation we panic if we incremented more than 64
    // bits allow.
    // In golang this would reset to an all zeroes nonce.
    let mut nonce = Nonce([0_u8, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255]);
    nonce.increment();
}
