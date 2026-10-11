// Generates the test vectors exactly the way the osmarks site does (see spec/mark-protocol-v0.md).
// Run: node test-vectors/generate.mjs   (Node 20+). Uses a throwaway key, never the real one.
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
const out = dirname(fileURLToPath(import.meta.url));
const enc = new TextEncoder();
const hex = (b) => [...new Uint8Array(b)].map((x) => x.toString(16).padStart(2, "0")).join("");
const b64u = (b) => Buffer.from(b).toString("base64url");
const sha256 = async (s) => hex(await crypto.subtle.digest("SHA-256", enc.encode(s)));
const kp = await crypto.subtle.generateKey({ name: "Ed25519" }, true, ["sign", "verify"]);
const jwk = await crypto.subtle.exportKey("jwk", kp.publicKey);
const sign = async (m) => b64u(await crypto.subtle.sign({ name: "Ed25519" }, kp.privateKey, enc.encode(m)));
const save = (f, o) => writeFileSync(join(out, f), JSON.stringify(o, null, 2) + "\n");

const people = [
  ["Example Ada", "Rebuilt the bakery's ordering page", "Corner Bakery", "2026-10-01", "confirmed", null],
  ["Example Ben", "Translated the onboarding guide into Spanish", "Example Co.", "2026-10-03", "amended", "It was Spanish and Portuguese."],
  ["Example Cy", "Edited a 12-minute product video", null, "2026-10-05", "confirmed", null],
];
const entries = [], proofs = [];
let prev = "0".repeat(64), t = Date.UTC(2026, 9, 6, 12, 0, 0);
for (let i = 0; i < people.length; i++) {
  const [by, statement, forWhom, delivered, verdict, correction] = people[i];
  t += 3600_000;
  const payload = JSON.stringify({ v: 1, mark: `00000000-0000-4000-8000-00000000000${i + 1}`, by, statement, for: forWhom,
    promised: null, delivered, work: null, verdict, correction, confirmer: "company-email", confirmed_at: new Date(t - 1000).toISOString() });
  const mark_hash = await sha256(payload);
  const signature = await sign(`${mark_hash}|${prev}|${t}`);
  entries.push({ seq: i + 1, mark_hash, prev_hash: prev, created_at: t, signature });
  proofs.push({ payload, mark_hash, log_seq: i + 1, prev_hash: prev, logged_at: t, signature, signed_message: `${mark_hash}|${prev}|${t}`, public_key: "/key" });
  prev = mark_hash;
}
const desc = "osmarks public append-only log. Each entry signs `mark_hash|prev_hash|created_at` with the key at /key. Contains no personal data.";
save("key.json", { alg: "Ed25519", public_jwk: jwk });
save("log.json", { description: desc, entries });
save("proof-2.json", proofs[1]);
// Tampered cases: each must FAIL verification.
save("bad-proof-payload-changed.json", { ...proofs[1], payload: proofs[1].payload.replace("Spanish", "French") });
save("bad-log-entry-removed.json", { description: desc, entries: [entries[0], entries[2]] });
save("bad-log-signature.json", { description: desc, entries: entries.map((e, i) => i === 1 ? { ...e, created_at: e.created_at + 1 } : e) });
console.log("written");
