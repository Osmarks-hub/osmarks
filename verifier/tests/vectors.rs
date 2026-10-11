use osmarks_verify::*;
use serde_json::Value;

fn load(name: &str) -> Value {
    let p = format!("{}/../test-vectors/{}", env!("CARGO_MANIFEST_DIR"), name);
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}
fn key() -> ed25519_dalek::VerifyingKey { key_from_json(&load("key.json")).unwrap() }

#[test]
fn good_log_verifies() {
    let log = log_from_json(&load("log.json")).unwrap();
    assert_eq!(verify_log(&key(), &log).unwrap(), 3);
}

#[test]
fn good_proof_verifies_alone_and_against_log() {
    let log = log_from_json(&load("log.json")).unwrap();
    let m = verify_mark(&key(), &load("proof-2.json"), None).unwrap();
    assert_eq!(m.log_seq, 2);
    verify_mark(&key(), &load("proof-2.json"), Some(&log)).unwrap();
}

#[test]
fn changed_payload_fails() {
    let e = verify_mark(&key(), &load("bad-proof-payload-changed.json"), None).unwrap_err();
    assert!(matches!(e, VerifyError::HashMismatch { .. }));
}

#[test]
fn removed_entry_fails() {
    let log = log_from_json(&load("bad-log-entry-removed.json")).unwrap();
    assert!(matches!(verify_log(&key(), &log).unwrap_err(), VerifyError::BrokenChain { seq: 3, .. }));
}

#[test]
fn backdated_entry_fails() {
    let log = log_from_json(&load("bad-log-signature.json")).unwrap();
    assert!(matches!(verify_log(&key(), &log).unwrap_err(), VerifyError::BadSignature { seq: Some(2) }));
}
