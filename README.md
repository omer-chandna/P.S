# P.S.

P.S. is a local-first prompt optimizer for developers. It uses a model running in [Ollama](https://ollama.com/download) to improve a prompt before you send it to a coding agent such as Cursor, Claude Code, or Codex.

It is a prompt optimizer, not a prompt expander. A better prompt is the smallest change that helps the next agent do the requested task. If the prompt is already clear, P.S. leaves it alone. If a critical decision is missing, P.S. asks one question instead of inventing the answer.

P.S. never submits the prompt for you.

## Platforms

| Platform | Support |
| --- | --- |
| macOS 14 Sonoma or newer, Apple Silicon | Menu bar app, ARM64 DMG |
| Windows 11, x64 | System tray app, NSIS installer |
| Linux | Not a product target. Useful for core tests and interface development |

Intel macOS and Windows 10 are out of scope.

## What a shortcut does

1. You select a prompt in another application.
2. You press the shortcut.
3. P.S. copies the selection, without deleting it.
4. One local model call returns `rewritten`, `unchanged`, or `needs_clarification`.
5. A validated rewrite replaces the selection only when that is safe.
6. Otherwise P.S. shows a review or a short notification and leaves the original text in place.

Default shortcuts:

| Action | macOS | Windows |
| --- | --- | --- |
| Improve | Control+Option+P | Ctrl+Alt+P |
| Compact | Control+Option+Shift+P | Ctrl+Alt+Shift+P |

Both can be changed or disabled. Closing the window hides P.S. Quit from the menu bar or tray.

## Privacy

- No account, subscription, or cloud inference.
- The Ollama address must be loopback (`127.0.0.1`, `localhost`, or `::1`).
- Prompt text is not written to disk or to logs.
- Model downloads happen only after you start them, and they contact Ollama’s registry.
- Cloud-hosted Ollama entries cannot be selected for inference.

## Develop

Requirements: Node.js 22, Rust 1.90, and a local Ollama server when you want real inference. Tests use mocked Ollama responses and do not download models.

```bash
npm install
cargo test -p ps-core
npm test
npm run tauri dev
```

The optimizer core lives in `crates/ps-core` and does not depend on Tauri. Prompt instructions are versioned in `prompts/`.

See `docs/DEVELOPMENT.md` for the roadmap, `docs/PROMPT_ENGINE.md` for the rewrite policy, and `docs/TESTING.md` for the manual checks that have to be done on macOS and Windows.

## Build an installer

Unsigned local builds:

```bash
npm run tauri build -- --bundles dmg --target aarch64-apple-darwin
npm run tauri build -- --bundles nsis --target x86_64-pc-windows-msvc
```

Signing and notarization are described in `docs/RELEASING.md`. They are not required to build.

## License

MIT. See [LICENSE](LICENSE).
