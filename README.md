# P.S.

P.S. is a local prompt optimizer for people who write prompts to coding agents.

It lives in the macOS menu bar or the Windows system tray. Select a prompt, press a shortcut, and a model running in [Ollama](https://ollama.com/download) rewrites it only when a clearer version would help the next agent. An already effective prompt stays exactly as you wrote it. If a decision only you can make is missing, P.S. asks one question. You still send the prompt yourself.

The rule is a **minimum sufficient prompt**: keep what changes the work, add only what helps it get done, and remove the rest. A better prompt is not a longer one. P.S. does not promise that the next agent will use fewer tokens.

![P.S. main window, with the manual optimizer and an empty result](docs/images/home.png)

## Use it

1. Select the prompt in Cursor, VS Code, a browser field, or another editor.
2. Press the shortcut.
3. Read the result, then submit it yourself.

| | macOS | Windows |
| --- | --- | --- |
| Improve | Control+Option+P | Ctrl+Alt+P |
| Compact | Control+Option+Shift+P | Ctrl+Alt+Shift+P |

Improve makes the request easier to carry out. Compact removes filler and keeps every requirement. Either shortcut can be changed or turned off in Settings.

Closing the window hides P.S. Quit from the menu bar or the tray.

### Three results

**Rewritten.** A checked improvement replaces the selection when the same window still has the same text. If focus moved, the selection changed, or the rewrite dropped something that mattered, P.S. opens a review instead of pasting.

**Unchanged.** The original text stays put. A small “Already clear” note is the whole result.

**Needs a decision.** P.S. shows one question in its own window. The question is not typed into your editor.

A vague bug report can become an instruction to investigate, fix, and check the result. A sentence that is already specific should survive untouched:

> Add a loading spinner to the Submit button while the existing request is in progress.

That is the behaviour P.S. is built for. It is not a promise that every model will phrase the rewrite the same way.

## Set up

Install [Ollama](https://ollama.com/download) and leave it running on this computer. The first launch of P.S. walks through the connection, a model, and the shortcut.

![Model manager with Fast, Balanced, and Quality suggestions](docs/images/models.png)

The suggestions are starting points, not a ranking:

| | Model | Aimed at |
| --- | --- | --- |
| Fast | `qwen3.5:2b` | Less memory, quicker rewrites |
| Balanced | `qwen3.5:4b` | The usual recommendation |
| Quality | `qwen3.5:9b` | Long prompts with many constraints |

Any compatible local model can be selected. Cloud entries are listed and cannot be used for inference. The download size on the card is not the memory the model needs while it is running.

P.S. runs on macOS 14 Sonoma or newer, Apple Silicon only, and on Windows 11 x64. Intel Macs and Windows 10 are outside the target. Linux is a development host, not a shipped app.

## Privacy

Inference stays on your machine. There is no account and no subscription. The Ollama address must be `localhost`, `127.0.0.1`, or `::1`. Selected prompts are not saved and are not written to logs. Starting a model download is the one action that contacts Ollama’s model registry.

## Develop

You need Node.js 22 and Rust 1.90. Tests mock Ollama and do not download models.

```bash
npm install
cargo test -p ps-core
npm test
npm run tauri dev
```

The optimizer is `crates/ps-core`. It does not depend on the desktop window, so a later CLI can use the same rules. The instructions sent to the model are versioned text in `prompts/`.

Build an unsigned installer from a Mac or a Windows machine:

```bash
npm run tauri build -- --bundles dmg --target aarch64-apple-darwin
npm run tauri build -- --bundles nsis --target x86_64-pc-windows-msvc
```

| | |
| --- | --- |
| [Prompt engine](docs/PROMPT_ENGINE.md) | What counts as a better prompt, and what the model is told |
| [Architecture](docs/ARCHITECTURE.md) | How capture, inference, and paste are split |
| [Testing](docs/TESTING.md) | Automated checks, and the manual pass still owed on macOS and Windows |
| [Releasing](docs/RELEASING.md) | Installers, signing, and notarization |
| [Contributing](CONTRIBUTING.md) | How to change the prompts without loosening the product rule |

## License

[MIT](LICENSE).
