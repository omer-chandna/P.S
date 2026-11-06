# Contributing

P.S. is a small desktop app with a strict product rule: preserve the user’s intent, and do not add instructions that do not help the next coding agent.

## Setup

```bash
npm install
cargo test -p ps-core
npm test
npm run lint
npm run typecheck
```

Use Rust 1.90, pinned in `rust-toolchain.toml`. Do not add a Linux product target, an Electron shell, a hosted backend, or a second model call for classification or critique.

## Changing prompts

Edit `prompts/shared.md`, `prompts/improve.md`, and `prompts/compact.md` together. Bump `ps-prompt-version` in each file. Describe the behaviour change in `docs/PROMPT_ENGINE.md`. The selected user text must stay in the user message, never inside the system prompt.

## Tests

Add a mocked Ollama case or a pure core test when you change parsing, preservation, delivery, or model recommendations. Do not download or rank real models in CI. Representative fixtures live in `fixtures/prompts.json`.

Native paste, accessibility, and installer checks are documented in `docs/TESTING.md`. Say explicitly when those were not run.

## Commits and pull requests

Use conventional commit subjects (`feat:`, `fix:`, `test:`, `docs:`, `ci:`). Keep a commit to one behaviour. In the pull request, list what you tested and which platforms you could not run.
