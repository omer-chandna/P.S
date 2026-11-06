# Install P.S.

P.S. version 0.1.0 is installed from this repository. You can run it in development, or build an unsigned installer and keep that. A signed download is a later release. Day-to-day use still needs [Ollama](https://ollama.com/download) on the same computer. The [README](../README.md#set-up) covers the model choices.

The desktop app runs on Apple Silicon macOS 14 Sonoma or newer, and on Windows 11 x64. Linux is a fine place to build and test the project. Capture, global shortcuts, and the installers are macOS and Windows.

## 1. Install the tools

On every machine:

| Tool | Version | Notes |
| --- | --- | --- |
| [Git](https://git-scm.com) | current | Used to clone the repository |
| [Node.js](https://nodejs.org) | 22 | Includes npm |
| [Rust](https://rustup.rs) | 1.90 | `rust-toolchain.toml` selects the compiler, rustfmt, and clippy when you use rustup |

On a Mac, install the Xcode command line tools:

```bash
xcode-select --install
```

On Windows 11, install the Visual Studio C++ build tools, and the WebView2 runtime if Windows has not already provided it. The NSIS package can download the WebView2 bootstrapper during install.

## 2. Clone the repository

```bash
git clone https://github.com/omer-chandna/P.S.git
cd P.S
npm install
```

`npm install` reads `package-lock.json` and installs the desktop shell. The first Rust build happens when you run or package the app.

## 3. Run it from source

From the repository, on a Mac or a Windows PC:

```bash
npm run tauri dev
```

The command compiles the app and opens the window. The interface is served at `http://127.0.0.1:1420` while that command is running. Leave the terminal open. Closing the window hides P.S. Quit from the menu bar on macOS, or from the system tray on Windows.

The first launch walks through six steps: welcome, the Ollama connection, a model, the shortcuts, a sample prompt, and finish. P.S. looks for Ollama at `http://127.0.0.1:11434`. Start Ollama before the connection step. `localhost` and `::1` are also accepted. The app will not install Ollama for you.

Onboarding can download a suggested local model. The usual starting point is `qwen3.5:4b`. Cloud model names are listed and refused for inference.

### macOS Accessibility

Copy and replace need Accessibility permission. In onboarding, choose **Open Accessibility settings**, then enable P.S. The development binary is named `postscript`, so look for that name if P.S. is absent from the list. After you enable it, return to P.S. and continue. Without the permission, the selection stays as you wrote it.

### Linux

On Linux, check the project with the commands in [Confirm a checkout](#confirm-a-checkout). The window can be built there for interface work. Replacing a selection in another app is a macOS and Windows feature, so a Linux run reports that platform as unsupported.

## 4. Build an installer

Build on the same operating system you are packaging. The artifacts are unsigned. macOS Gatekeeper and Windows SmartScreen ask for confirmation before an unsigned app opens. Signing is described in [Releasing](RELEASING.md).

The bundle directory is the repository `target` folder. It is not `src-tauri/target`.

### macOS

```bash
npm run tauri build -- --bundles dmg --target aarch64-apple-darwin
```

The disk image is `target/aarch64-apple-darwin/release/bundle/dmg/PS_0.1.0_aarch64.dmg`. Open it and move **PS** to Applications. The window title remains P.S. The bundle identifier is `app.postscript.ps`, and the minimum system version is macOS 14.

After installing, grant Accessibility to PS the same way as for a development build.

### Windows 11

```bash
npm run tauri build -- --bundles nsis --target x86_64-pc-windows-msvc
```

The installer is `target/x86_64-pc-windows-msvc/release/bundle/nsis/PS_0.1.0_x64-setup.exe`. It installs for the current user and includes an uninstaller.

## 5. Update

From a development checkout:

```bash
git pull
npm install
npm run tauri dev
```

For an installed copy, pull those same commits and build the installer again, then run the new DMG or setup program.

## Confirm a checkout

These are the checks the project runs on a pull request. They use a mocked Ollama server. Installed models stay untouched, and the Ollama daemon can stay stopped.

```bash
cargo fmt -p ps-core --all -- --check
cargo clippy -p ps-core --all-targets -- -D warnings
cargo test -p ps-core
npm run lint
npm run typecheck
npm test
npm run build
```

`cargo test -p ps-core` is enough when you only want the optimizer tests. `npm test` covers the window.
