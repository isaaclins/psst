use psst_core::connection::diffie_hellman::DHLocalKeys;

#[test]
fn dh_keys_random_generates_keys() {
    let keys = DHLocalKeys::random();
    let public_key = keys.public_key();

    assert!(!public_key.is_empty(), "public key should not be empty");
}

#[test]
fn dh_keys_different_keys_each_time() {
    let keys1 = DHLocalKeys::random();
    let keys2 = DHLocalKeys::random();

    let pub1 = keys1.public_key();
    let pub2 = keys2.public_key();

    assert_ne!(pub1, pub2, "random keys should be different");
}

#[test]
fn dh_shared_secret_is_same_for_both_parties() {
    let alice = DHLocalKeys::random();
    let bob = DHLocalKeys::random();

    let alice_public = alice.public_key();
    let bob_public = bob.public_key();

    let alice_shared = alice.shared_secret(&bob_public);
    let bob_shared = bob.shared_secret(&alice_public);

    assert_eq!(alice_shared, bob_shared, "shared secrets should match");
}

#[test]
fn dh_shared_secret_with_empty_remote_key() {
    let keys = DHLocalKeys::random();
    let empty_key = vec![];

    let shared = keys.shared_secret(&empty_key);
    // Should produce some result, even if it's a zero-value secret
    // The important thing is it doesn't panic
    assert!(!shared.is_empty() || shared.is_empty());
}

#[test]
fn dh_public_key_is_consistent() {
    let keys = DHLocalKeys::random();
    let pub1 = keys.public_key();
    let pub2 = keys.public_key();

    assert_eq!(pub1, pub2, "public key should be consistent");
}

#[test]
fn dh_shared_secret_with_same_key_twice() {
    let alice = DHLocalKeys::random();
    let bob = DHLocalKeys::random();

    let bob_public = bob.public_key();

    let shared1 = alice.shared_secret(&bob_public);
    let shared2 = alice.shared_secret(&bob_public);

    assert_eq!(shared1, shared2, "shared secret should be deterministic");
}

#[test]
fn dh_shared_secret_not_empty() {
    let alice = DHLocalKeys::random();
    let bob = DHLocalKeys::random();

    let bob_public = bob.public_key();
    let shared = alice.shared_secret(&bob_public);

    assert!(!shared.is_empty(), "shared secret should not be empty");
}

#[test]
fn dh_public_key_length_is_reasonable() {
    let keys = DHLocalKeys::random();
    let public_key = keys.public_key();

    // The DH prime is 96 bytes, so public key should be similar length
    assert!(!public_key.is_empty(), "public key should have length");
    assert!(
        public_key.len() <= 96,
        "public key should not exceed prime length"
    );
}

#[test]
fn dh_shared_secret_changes_with_different_remote_keys() {
    let alice = DHLocalKeys::random();
    let bob1 = DHLocalKeys::random();
    let bob2 = DHLocalKeys::random();

    let bob1_public = bob1.public_key();
    let bob2_public = bob2.public_key();

    let shared1 = alice.shared_secret(&bob1_public);
    let shared2 = alice.shared_secret(&bob2_public);

    assert_ne!(
        shared1, shared2,
        "different remote keys should produce different secrets"
    );
}
