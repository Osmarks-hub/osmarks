use osmarks_verify::{key_from_json, log_from_json, verify_log, verify_mark};
use serde_json::Value;
use std::process::ExitCode;

const USAGE: &str = "osmarks-verify: check osmarks marks without trusting osmarks.

Usage:
  osmarks-verify log  <log.json>  --key <key.json>
  osmarks-verify mark <proof.json> --key <key.json> [--log <log.json>]

Get the files from an osmarks site:
  key.json    https://<site>/key
  log.json    https://<site>/log
  proof.json  https://<site>/m/<id>.json

Exit code 0 means verified; 1 means it failed; 2 means the command was used wrongly.";

fn read_json(path: &str) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path} is not valid JSON: {e}"))
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn run(args: &[String]) -> Result<String, (u8, String)> {
    let usage = || (2u8, USAGE.to_string());
    let cmd = args.get(1).ok_or_else(usage)?;
    let target = args.get(2).filter(|a| !a.starts_with("--")).ok_or_else(usage)?;
    let key_path = flag(args, "--key").ok_or_else(usage)?;
    let fail = |m: String| (1u8, m);

    let key = key_from_json(&read_json(&key_path).map_err(fail)?).map_err(|e| fail(e.to_string()))?;
    match cmd.as_str() {
        "log" => {
            let entries = log_from_json(&read_json(target).map_err(fail)?).map_err(|e| fail(e.to_string()))?;
            let n = verify_log(&key, &entries).map_err(|e| fail(e.to_string()))?;
            Ok(format!("Verified: all {n} log entries are signed and correctly chained."))
        }
        "mark" => {
            let proof = read_json(target).map_err(fail)?;
            let log = match flag(args, "--log") {
                Some(p) => {
                    let entries = log_from_json(&read_json(&p).map_err(fail)?).map_err(|e| fail(e.to_string()))?;
                    verify_log(&key, &entries).map_err(|e| fail(format!("the log itself failed: {e}")))?;
                    Some(entries)
                }
                None => None,
            };
            let m = verify_mark(&key, &proof, log.as_deref()).map_err(|e| fail(e.to_string()))?;
            let p = &m.payload;
            let s = |k: &str| p.get(k).and_then(Value::as_str).unwrap_or("-").to_string();
            let mut out = format!(
                "Verified: mark #{} is unchanged and signed.\n  statement: {}\n  by: {}\n  delivered: {}\n  verdict: {}\n  confirmed at: {}",
                m.log_seq, s("statement"), s("by"), s("delivered"), s("verdict"), s("confirmed_at")
            );
            if log.is_some() {
                out.push_str("\n  It matches the public log you provided.");
            }
            Ok(out)
        }
        _ => Err(usage()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err((code, msg)) => {
            eprintln!("{msg}");
            ExitCode::from(code)
        }
    }
}
