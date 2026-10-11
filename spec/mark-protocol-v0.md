# osmarks mark protocol, v0

*Status: in use on osmarks since 2026-10-09. License: CC BY-SA 4.0.*

A **mark** is a record that someone did a piece of work, confirmed by a witness who saw it. This document describes exactly how a mark is fingerprinted, signed and logged, so that anyone can check a mark without trusting osmarks.

## 1. The payload

When a witness answers a confirmation request, the site builds the mark's **payload**: a JSON object serialized to a single string. The string is stored and published exactly as built; verifiers must use it byte for byte and must not re-serialize it.

Fields (v1):

| Field | Meaning |
|---|---|
| `v` | Payload version, currently `1` |
| `mark` | The mark's id |
| `by` | The name the person gave for themselves |
| `statement` | What they did, in their words |
| `for` | Who it was for (optional, free text) |
| `promised` | Date they had promised it by (optional, `YYYY-MM-DD`) |
| `delivered` | Date delivered (`YYYY-MM-DD`) |
| `work` | Link to the work (optional) |
| `verdict` | `confirmed`, `amended` (confirmed with a correction) or `declined` |
| `correction` | The witness's correction, when `amended` |
| `confirmer` | `company-email` or `personal-email`: how the witness proved who they are |
| `confirmed_at` | ISO 8601 time of the witness's answer |

The payload contains no email addresses. The witness's identity is checked by a one-time code sent to their email; osmarks keeps only a salted hash of that address.

## 2. The fingerprint

`mark_hash` = lowercase hex of SHA-256 over the UTF-8 bytes of the payload string.

## 3. The public log

Every answered confirmation, including declined ones, is appended to a public, append-only log. Each entry is:

| Field | Meaning |
|---|---|
| `seq` | Position in the log, strictly increasing |
| `mark_hash` | The mark's fingerprint |
| `prev_hash` | The `mark_hash` of the entry before it; the first entry uses 64 zeros |
| `created_at` | Time appended, milliseconds since the Unix epoch |
| `signature` | Ed25519 signature, base64url without padding |

The signed message is the UTF-8 string:

```
<mark_hash>|<prev_hash>|<created_at>
```

Because each entry includes the previous entry's fingerprint and is signed, removing, inserting, reordering or back-dating an entry breaks the chain or a signature. The log holds fingerprints only, no personal data.

## 4. Where to get things

On an osmarks site:

| Path | Content |
|---|---|
| `/key` | `{"alg":"Ed25519","public_jwk":{...}}` — the public key (JWK, `crv` `Ed25519`, `x` base64url) |
| `/log` | `{"description":..., "entries":[...]}` — the whole log |
| `/m/<id>.json` | One mark's proof: `payload`, `mark_hash`, `log_seq`, `prev_hash`, `logged_at`, `signature` |
| `/m/<id>` | The same mark, readable by people |

Declined marks appear in the log as fingerprints only; their pages are not public.

## 5. How to verify

**A single mark:**
1. Compute SHA-256 of `payload`; it must equal `mark_hash`.
2. Verify `signature` over `mark_hash|prev_hash|logged_at` with the public key.
3. Optionally, find `mark_hash` in the log and check that `seq`, `prev_hash`, `created_at` and `signature` match the proof.

**The whole log:**
1. Walk entries in `seq` order, starting from `prev_hash` = 64 zeros.
2. Each entry's `prev_hash` must equal the previous entry's `mark_hash`.
3. Each entry's signature must verify.

The reference implementation is `verifier/` in this repository. Test vectors, including tampered cases that must fail, are in `test-vectors/`.

## 6. Keeping verification possible without osmarks

Anyone may copy and mirror the key and the log. With a copy of both, every mark can still be verified even if osmarks disappears. Planned: periodic anchoring of the log's latest fingerprint to a public blockchain as a timestamp (no token is involved).

## 7. Open questions

- Key rotation: how a new key is announced and how old marks stay verifiable.
- Publishing the payload as a W3C Verifiable Credential in addition to the current form.
- A canonical serialization, so that future implementations can rebuild the payload instead of storing it verbatim.
