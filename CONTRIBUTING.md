# Contributing to osmarks

Thank you for wanting to help. osmarks is built to be owned in part by the people who build it.

## What helps most right now

- **Verifier**: a WebAssembly build so marks can be checked in the browser; bindings for JavaScript and Python.
- **Profile validator**: checks that a profile follows `spec/profile-format-v0.md`, that every line has a source tag, and flags text that tries to give instructions to an AI.
- **MCP server for profiles**: lets MCP-capable AI tools read and update a person's profile files directly.
- **Evaluation harness**: runs `guides/` across several AI models with simulated people and scores how natural, accurate and kind the conversation was.
- **Translations** of the guides, done carefully.

## How

1. Open an issue first for anything bigger than a small fix, so we agree on the approach.
2. Keep pull requests small and focused. For code, add tests; the `test-vectors/` must keep passing.
3. Changes to `spec/` need a short rationale and must say what happens to existing marks and profiles.

## Contribution points

osmarks records contributions as **contribution points** under published rules (the member co-ownership plan). Accepted code, specs, reviews and translations count. Points:

- cannot be bought or transferred, and carry no promise of cash value;
- are recorded publicly under rules published in advance;
- will be converted into any form of ownership or revenue share only after legal review, under terms published beforehand.

## Ground rules

- Be kind and direct. Critique the work, not the person.
- Never put real people's personal data in issues, tests or examples.
- Security problems: email marks@osmarks.com instead of opening a public issue.

## Licenses

By contributing you agree that your contribution is licensed under the license of the folder it goes into: CC BY-SA 4.0 for `guides/` and `spec/`, Apache-2.0 for code and `test-vectors/`.
