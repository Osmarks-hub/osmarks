//! osmarks-verify: check osmarks marks and the public log without trusting osmarks.
//!
//! Everything here follows `spec/mark-protocol-v0.md`. In short:
//! - A mark's `mark_hash` is the lowercase hex SHA-256 of its exact payload string.
//! - Each public log entry signs the UTF-8 message `mark_hash|prev_hash|created_at`
//!   with the osmarks Ed25519 key.
//! - `prev_hash` is the previous entry's `mark_hash`; the first entry uses 64 zeros.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;

pub const GENESIS_PREV: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, PartialEq)]
pub enum VerifyError {
    BadKey(String),
    BadInput(String),
    HashMismatch { expected: String, actual: String },
    BadSignature { seq: Option<i64> },
    BrokenChain { seq: i64, expected_prev: String, actual_prev: String },
    OutOfOrder { seq: i64 },
    NotInLog { mark_hash: String },
    LogMismatch { seq: i64, field: &'static str },
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerifyError::BadKey(m) => write!(f, "the public key could not be read: {m}"),
            VerifyError::BadInput(m) => write!(f, "the input could not be read: {m}"),
            VerifyError::HashMismatch { expected, actual } => write!(
                f,
                "the payload does not match its fingerprint (claimed {expected}, computed {actual}); the mark was changed"
            ),
            VerifyError::BadSignature { seq: Some(s) } => write!(f, "log entry #{s} has an invalid signature"),
            VerifyError::BadSignature { seq: None } => write!(f, "the signature is invalid"),
            VerifyError::BrokenChain { seq, expected_prev, actual_prev } => write!(
                f,
                "log entry #{seq} does not follow the entry before it (expected prev_hash {expected_prev}, found {actual_prev}); an entry was removed, inserted or changed"
            ),
            VerifyError::OutOfOrder { seq } => write!(f, "log entry #{seq} is out of order"),
            VerifyError::NotInLog { mark_hash } => write!(f, "mark {mark_hash} is not in the given log"),
            VerifyError::LogMismatch { seq, field } => {
                write!(f, "the proof's {field} differs from log entry #{seq}")
            }
        }
    }
}

impl std::error::Error for VerifyError {}

/// A single entry of the public log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub seq: i64,
    pub mark_hash: String,
    pub prev_hash: String,
    pub created_at: i64,
    pub signature: String,
}

/// Lowercase hex SHA-256 of a UTF-8 string.
pub fn sha256_hex(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

/// The exact message a log entry signs.
pub fn signed_message(mark_hash: &str, prev_hash: &str, created_at: i64) -> String {
    format!("{mark_hash}|{prev_hash}|{created_at}")
}

/// Read the key published at `/key`: `{"alg":"Ed25519","public_jwk":{"kty":"OKP","crv":"Ed25519","x":"..."}}`.
/// A bare JWK object is accepted too.
pub fn key_from_json(v: &Value) -> Result<VerifyingKey, VerifyError> {
    let jwk = v.get("public_jwk").unwrap_or(v);
    let crv = jwk.get("crv").and_then(Value::as_str).unwrap_or("");
    if crv != "Ed25519" {
        return Err(VerifyError::BadKey(format!("expected an Ed25519 key, found crv \"{crv}\"")));
    }
    let x = jwk
        .get("x")
        .and_then(Value::as_str)
        .ok_or_else(|| VerifyError::BadKey("missing \"x\"".into()))?;
    let bytes = URL_SAFE_NO_PAD
        .decode(x)
        .map_err(|e| VerifyError::BadKey(format!("\"x\" is not base64url: {e}")))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| VerifyError::BadKey("\"x\" must be 32 bytes".into()))?;
    VerifyingKey::from_bytes(&arr).map_err(|e| VerifyError::BadKey(e.to_string()))
}

fn verify_sig(key: &VerifyingKey, message: &str, sig_b64u: &str, seq: Option<i64>) -> Result<(), VerifyError> {
    let raw = URL_SAFE_NO_PAD
        .decode(sig_b64u)
        .map_err(|_| VerifyError::BadSignature { seq })?;
    let arr: [u8; 64] = raw.try_into().map_err(|_| VerifyError::BadSignature { seq })?;
    key.verify(message.as_bytes(), &Signature::from_bytes(&arr))
        .map_err(|_| VerifyError::BadSignature { seq })
}

/// Verify one log entry's signature.
pub fn verify_entry(key: &VerifyingKey, e: &LogEntry) -> Result<(), VerifyError> {
    verify_sig(key, &signed_message(&e.mark_hash, &e.prev_hash, e.created_at), &e.signature, Some(e.seq))
}

fn str_field(v: &Value, k: &str) -> Result<String, VerifyError> {
    v.get(k)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| VerifyError::BadInput(format!("missing text field \"{k}\"")))
}

fn int_field(v: &Value, k: &str) -> Result<i64, VerifyError> {
    v.get(k)
        .and_then(Value::as_i64)
        .ok_or_else(|| VerifyError::BadInput(format!("missing number field \"{k}\"")))
}

/// Read the log published at `/log`: `{"entries":[{seq, mark_hash, prev_hash, created_at, signature}]}`.
pub fn log_from_json(v: &Value) -> Result<Vec<LogEntry>, VerifyError> {
    let arr = v
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| VerifyError::BadInput("missing \"entries\" list".into()))?;
    arr.iter()
        .map(|e| {
            Ok(LogEntry {
                seq: int_field(e, "seq")?,
                mark_hash: str_field(e, "mark_hash")?,
                prev_hash: str_field(e, "prev_hash")?,
                created_at: int_field(e, "created_at")?,
                signature: str_field(e, "signature")?,
            })
        })
        .collect()
}

/// Verify the whole log: every signature, and that each entry chains to the one before it.
/// Returns the number of entries checked.
pub fn verify_log(key: &VerifyingKey, entries: &[LogEntry]) -> Result<usize, VerifyError> {
    let mut prev = GENESIS_PREV.to_string();
    let mut last_seq = i64::MIN;
    for e in entries {
        if e.seq <= last_seq {
            return Err(VerifyError::OutOfOrder { seq: e.seq });
        }
        if e.prev_hash != prev {
            return Err(VerifyError::BrokenChain { seq: e.seq, expected_prev: prev, actual_prev: e.prev_hash.clone() });
        }
        verify_entry(key, e)?;
        prev = e.mark_hash.clone();
        last_seq = e.seq;
    }
    Ok(entries.len())
}

/// What a verified mark says, for display.
#[derive(Debug)]
pub struct VerifiedMark {
    pub mark_hash: String,
    pub log_seq: i64,
    pub payload: Value,
}

/// Verify a single mark proof as published at `/m/<id>.json`.
/// If `log` is given, also check the proof matches that log entry exactly.
pub fn verify_mark(key: &VerifyingKey, proof: &Value, log: Option<&[LogEntry]>) -> Result<VerifiedMark, VerifyError> {
    let payload = str_field(proof, "payload")?;
    let mark_hash = str_field(proof, "mark_hash")?;
    let prev_hash = str_field(proof, "prev_hash")?;
    let logged_at = int_field(proof, "logged_at")?;
    let seq = int_field(proof, "log_seq")?;
    let signature = str_field(proof, "signature")?;

    let computed = sha256_hex(&payload);
    if computed != mark_hash {
        return Err(VerifyError::HashMismatch { expected: mark_hash, actual: computed });
    }
    verify_sig(key, &signed_message(&mark_hash, &prev_hash, logged_at), &signature, Some(seq))?;

    if let Some(entries) = log {
        let e = entries
            .iter()
            .find(|e| e.mark_hash == mark_hash)
            .ok_or_else(|| VerifyError::NotInLog { mark_hash: mark_hash.clone() })?;
        if e.seq != seq {
            return Err(VerifyError::LogMismatch { seq: e.seq, field: "log_seq" });
        }
        if e.prev_hash != prev_hash {
            return Err(VerifyError::LogMismatch { seq: e.seq, field: "prev_hash" });
        }
        if e.created_at != logged_at {
            return Err(VerifyError::LogMismatch { seq: e.seq, field: "logged_at" });
        }
        if e.signature != signature {
            return Err(VerifyError::LogMismatch { seq: e.seq, field: "signature" });
        }
    }

    let parsed: Value = serde_json::from_str(&payload)
        .map_err(|e| VerifyError::BadInput(format!("payload is not JSON: {e}")))?;
    Ok(VerifiedMark { mark_hash, log_seq: seq, payload: parsed })
}
