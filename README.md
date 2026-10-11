# osmarks

**Everyone's story deserves to be taken seriously.**

osmarks helps people get to know themselves with an AI companion called Margin, build a personal system they own, and leave marks worth keeping: the good they do, recorded and confirmed by the people who saw it.

This repository is the open part of osmarks: the guides anyone can paste into any AI, the file formats, and tools anyone can use to check our work without trusting us. Site: [osmarks.marks-90a.workers.dev](https://osmarks.marks-90a.workers.dev)

## What's here

| Folder | What it is | License |
|---|---|---|
| `guides/` | Plain-text guides for any AI: the first conversation with Margin (English and Chinese), daily use, the take-away file | CC BY-SA 4.0 |
| `spec/` | The profile format and the mark protocol | CC BY-SA 4.0 |
| `verifier/` | `osmarks-verify`, a small Rust tool that checks a mark or the whole public log offline | Apache-2.0 |
| `test-vectors/` | Known-good and deliberately tampered examples every implementation must agree on | Apache-2.0 |

## Check a mark yourself

```sh
cd verifier
cargo build --release
curl -s https://osmarks.marks-90a.workers.dev/key > key.json
curl -s https://osmarks.marks-90a.workers.dev/log > log.json
./target/release/osmarks-verify log log.json --key key.json
./target/release/osmarks-verify mark proof.json --key key.json --log log.json   # proof.json from /m/<id>.json
```

A mark's fingerprint, its place in the log and the time are signed with the osmarks key. If anyone changes, removes or back-dates a mark, the check fails. Keep your own copy of the key and the log, and you can verify marks even if osmarks disappears.

## Principles

- **Your file, your life.** Your profile is plain text and belongs to you. Paste it into any AI.
- **Text for what's yours, signatures for what others vouch for.** You may edit your own profile freely. A mark someone confirmed cannot be changed by anyone, including us.
- **Open method, attributed.** Use and adapt the guides and formats freely; credit osmarks and share your changes under the same license.
- **No tricks.** No streaks, no guilt, no selling in the conversation.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Good first projects: a profile validator, a browser build of the verifier (WebAssembly), an MCP server for profiles, and an evaluation harness that runs the guides across different AI models.

---

## 中文简介

**每个人的故事都值得被认真对待。**

osmarks 帮人和 AI 陪伴者 Margin（页边）一起认识自己，搭一套归自己所有的体系，并留下值得回看的印记：你做成的好事被记下来，并由见证的人确认。

这个仓库是 osmarks 公开的部分：任何人都能粘贴进任何 AI 的指南、档案格式和印记协议，以及一个不需要信任我们、就能自己核对印记真假的工具。

- **指南和规范**以 CC BY-SA 4.0 许可公开：可以用、可以改，但要署名 osmarks，改过的版本也要用同样的许可公开。
- **代码**以 Apache-2.0 许可公开。
- 欢迎懂技术的朋友一起建设。贡献按公开规则记入 osmarks 的会员共有积分，详见 CONTRIBUTING.md。
