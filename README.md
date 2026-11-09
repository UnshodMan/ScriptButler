# ScriptButler

> **提示**：中文版请见 [doc/README.zh-CN.md](doc/README.zh-CN.md)。

A desktop script management app built with **Tauri v2 + Vue 3 + TypeScript**. It keeps your local scripts organized as cards and supports drag-and-drop import, sorting by recent use, syntax-highlighted editing, and customizable, auto-saved run command schemes.

## Features

- **Card-based main view**: Scripts are arranged in a single vertical column of cards (the default window width fits exactly one card). The taller, fuller cards show the language tag, file name, execution directory, last used time, and usage count.
- **Quick run**: Each card has a "Run" button on the right that runs the script with the currently active scheme and opens the detail page to show the output; hovering over the button shows the full command that will be executed.
- **Drag-and-drop import**: Drag script files straight into the window to add cards (the "Add Script" file picker in the top-right corner works too). Dropping the same file again moves it to the top instead of adding a duplicate.
- **Search and filtering**: The search box at the top of the main view returns results in priority order: file name → directory → content. Name and directory matches are grouped in real time by Rust over the sorted entries, while content search scans on demand after typing pauses (using the system's ripgrep / grep when available and falling back to reading files one by one in Rust). No index needs to be built in advance.
- **Script hosting**: The "Host" button on a card copies the script into `scripts/` under the app data directory (the exact path is resolved by Tauri's `appDataDir` per platform). The copy is named with a timestamp and keeps the original extension. After hosting, the button icon becomes ❒ in orange and is no longer clickable (tooltip: "Hosted"). The card then shows only the effective execution directory, and only when the scheme has "Change to the script's directory before running" enabled (hosted scripts use a configurable execution directory, regular scripts use their own directory). The original script name, directory information, and run schemes are preserved, and the execution directory can be changed at any time on the detail page.
- **MRU sorting**: Cards are sorted by last used → usage count → name by default.
- **Syntax-highlighted editing**: Click a card to open the detail page, which provides highlighting and keyword completion powered by CodeMirror 6 and detects the language from the file extension (Python / JavaScript / TypeScript / Shell / JSON / YAML / Markdown / HTML / CSS, etc.). Edits can be written back to the file via the "Save" button or `Ctrl/Cmd + S`.
- **Custom run schemes**: The detail page header offers a run button and a scheme dropdown. Each script maintains its own set of run command schemes with create / edit / delete support; command templates support the `{file}` placeholder (the script path is appended at the end automatically when omitted). Each scheme includes a "Change to the script's directory before running" toggle (on by default); when enabled, the actual command looks like `cd '<script directory>' && <command>`, which avoids failures for scripts that depend on the current working directory. Scheme changes are saved instantly and available from the dropdown the next time the script is opened; a default scheme is generated based on the language on first open. The full command executed by the current scheme is shown below the run button.
- **Run output panel**: Streams stdout / stderr in real time (stderr highlighted in red) along with the exit code, with support for stopping mid-run and clearing the output.
- **Persistence**: Script entries and run schemes are stored in `scripts.json` in the app data directory and restored after a restart.
- **Internationalization**: Ships with 9 UI languages (简体中文 / 繁體中文 / English / 日本語 / 한국어 / Español / Français / Deutsch / Русский). The app follows the system language by default, and the language can be switched at any time from the top-right corner of the main and detail pages; the choice is remembered.
- **Agent integration**: Ships the `sbctl` CLI with the desktop app. Agents and terminals can add, manage, list, search, run, and remove scripts with stable `--json` output, and `sbctl skill --install <dir>` installs a ready-to-use Agent Skill. See [Agent integration](#agent-integration-cli--skill).

## Tech stack

| Layer             | Technology                        |
| ----------------- | --------------------------------- |
| Desktop framework | Tauri v2                          |
| Frontend          | Vue 3 + TypeScript + Vite + Pinia |
| Editor            | CodeMirror 6 (`one-dark` theme) |
| Plugin            | `tauri-plugin-dialog`           |
| CLI               | Rust (`scriptbutler-core` + `clap`) |

> Architecture convention: **business logic lives in the shared Rust core crate** (`src-tauri/core`), used by both the desktop app and the `sbctl` CLI. The frontend only calls these capabilities through Tauri IPC commands and events and renders the UI.

## Project structure

```
src/                  # Vue frontend (UI rendering and IPC calls only)
  lib/                # api / language / runner / editor / toast / time / i18n
  locales/            # 9 language packs (zh-CN is the baseline; other languages stay in sync with it)
  stores/scripts.ts   # Pinia store (thin cache of the backend entry list)
  components/         # CardGrid / ScriptCard / ScriptDetail / RunBar / SchemeDialog / OutputPanel / LangSwitcher
src-tauri/            # Tauri desktop app (IPC commands + child process events)
  src/commands.rs     # Thin IPC command wrappers over the shared core
  src/state.rs        # In-memory entries + scripts.json reload/save
  src/runner.rs       # Child process startup, output events, stop
  core/               # scriptbutler-core: models / store / ops / exec / search / language
  cli/                # scriptbutler-cli: sbctl command-line interface
  nsis/               # Windows installer template with the sbctl PATH option
  capabilities/       # core / dialog permissions
skills/               # Agent Skill (script-butler/SKILL.md)
scripts/              # build-cli.mjs / bump-version.mjs
```

## Development

Prerequisites: Node.js, the Rust toolchain, and Tauri's system dependencies (on Linux, `webkit2gtk-4.1` and others).

```bash
npm install         # Install dependencies
npm run build:cli   # Build the sbctl sidecar (also runs automatically before tauri dev/build)
npm run tauri dev   # Start the desktop app (development mode)
```

## Build

```bash
npm run build        # Frontend type check + bundling (vue-tsc + vite)
npm run build:cli    # Build the sbctl sidecar only
npm run tauri build  # Build the desktop installer (bundles sbctl)
```

The bundler picks up `src-tauri/binaries/sbctl-<target-triple>` through `bundle.externalBin`. The `beforeBuildCommand` builds it for the target triple that Tauri is using (`TAURI_ENV_TARGET_TRIPLE`), and CI pre-builds it for each matrix target.

### Version bumping

The app version is mirrored in `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`. The workspace version in `[workspace.package]` is inherited by `scriptbutler-core` and `scriptbutler-cli`; the script updates it and all three lockfile entries. Bump everything in one step:

```bash
npm run version:bump -- patch   # 0.1.0 -> 0.1.1
npm run version:bump -- minor   # 0.1.0 -> 0.2.0
npm run version:bump -- major   # 0.1.0 -> 1.0.0
npm run version:bump -- 0.2.3   # set an explicit version
```

### Continuous integration

Pushing a `v*` tag or manually triggering the `Build` workflow (`.github/workflows/build.yml`) builds installers for each platform in parallel on GitHub Actions:

| Platform | Artifacts                           |
| -------- | ----------------------------------- |
| Linux    | `.deb` / `.rpm` / `.AppImage` |
| Windows  | NSIS installer (MSI is not built)   |
| macOS    | `.dmg` for arm64 and x86_64       |

When triggered by a tag, the artifacts are attached to the corresponding draft release; when triggered manually, they can be downloaded from the Artifacts of the workflow run. The Windows installer wizard follows the system language (all 9 languages built into the app are supported) and falls back to English when the system language is not supported.

Each matrix job also publishes the `sbctl` binary it builds for the installer as a standalone release asset (and as a workflow artifact): `sbctl-linux-x86_64`, `sbctl-windows-x86_64.exe`, `sbctl-macos-aarch64`, and `sbctl-macos-x86_64`.

## Agent integration (CLI + Skill)

The installer ships `sbctl`, a Rust CLI that operates on the same `scripts.json` as the desktop app. It works whether or not the app is running.

### Binary location and PATH

| Platform | Binary location | PATH |
| --- | --- | --- |
| Linux (deb/rpm) | `/usr/bin/sbctl` | Added automatically |
| Windows (NSIS) | `<install dir>\sbctl.exe` | Check "Add sbctl to PATH" in the installer, or run `sbctl env add` |
| macOS (dmg) | `ScriptButler.app/Contents/MacOS/sbctl` | Run `/Applications/ScriptButler.app/Contents/MacOS/sbctl env add`; add `~/.local/bin` to PATH if prompted |
| Linux (AppImage) | Inside the AppImage | Not exposed; use the deb/rpm package |

`sbctl env add` on Windows adds the executable directory to the current user's PATH (use `--system` for the machine PATH). On macOS/Linux it creates `~/.local/bin/sbctl` unless the directory is already on PATH. `sbctl env remove` undoes it.

### Standalone CLI without the desktop app

Releases also ship the raw CLI binaries, so you can use `sbctl` without installing the desktop app:

```bash
# Linux / macOS
chmod +x sbctl-linux-x86_64          # sbctl-macos-aarch64 / sbctl-macos-x86_64 on macOS
./sbctl-linux-x86_64 env add         # links ~/.local/bin/sbctl and reports PATH status
```

On Windows, rename `sbctl-windows-x86_64.exe` to `sbctl.exe` and run `sbctl env add` (it adds the executable's directory to PATH). The CLI resolves the data directory itself and creates `scripts.json` on the first write, so it works on a machine where the desktop app was never installed. macOS may block the first run of the unsigned binary; allow it under System Settings → Privacy & Security.

### Commands

| Command | Description |
| --- | --- |
| `sbctl add <path> [--name N] [--command TPL] [--scheme-name N] [--json]` | Register a script in place; re-adding only updates MRU |
| `sbctl manage <path> [--name N] [--workdir DIR] [--command TPL] [--scheme-name N] [--json]` | Copy a script into the managed directory; works for registered and unregistered files |
| `sbctl remove <name\|path> [--json]` | Remove an entry; never deletes files |
| `sbctl list [--query KW] [--json]` | List entries in MRU order |
| `sbctl show <name\|path> [--json]` | Show an entry, its schemes, and the effective command |
| `sbctl search <keyword> [--content] [--json]` | Search names, directories, and optionally contents |
| `sbctl run <name\|path> [--scheme ID\|NAME] [--json]` | Run the active or selected scheme |
| `sbctl path [--json]` | Print the data directory, store path, managed directory, and executable |
| `sbctl env add\|remove\|status [--system] [--json]` | Manage PATH |
| `sbctl skill [--install <skills-root>]` | Print or install the Agent Skill |

`<name|path>` resolves by absolute path, then by the managed script's source path, then by case-insensitive file name. Duplicate names return an `ambiguous` error with the candidate paths.

### JSON output and exit codes

Pass `--json` for machine-readable output:

```json
{ "ok": true, "data": [ ... ] }
{ "ok": false, "error": { "code": "not_found", "message": "..." } }
```

`add`/`manage` also return `created` and `copiedTo`; `run` returns `{ exitCode, stdout, stderr }`.

| Exit code | Meaning |
| --- | --- |
| 0 | success |
| 1 | runtime error (I/O, spawn, manage) |
| 2 | not found, ambiguous reference, or usage error |
| 3 | invalid `scripts.json` |
| script code | `sbctl run` returns the script's exit code |

### Agent Skill

The repository contains `skills/script-butler/SKILL.md`. Install it into your agent's skills directory with:

```bash
sbctl skill --install ~/.claude/skills   # use your agent's skills root
```

`sbctl skill` prints the same document to stdout.

### Data synchronization

The CLI reads and writes the same `scripts.json` as the app. The app reloads the file before each command when it changed on disk and refreshes the card list when the window regains focus. If both processes write at the same time, the last writer wins.

## Data storage

Script entries and run schemes are read from and written to `scripts.json` in the app data directory by the shared Rust core (in the format `{ "entries": [...] }`). The `sbctl` CLI uses the same path; set `SCRIPTBUTLER_DATA_DIR` to override the CLI data directory (useful for tests):

- Linux: `~/.local/share/com.scriptbutler.app/scripts.json`
- macOS: `~/Library/Application Support/com.scriptbutler.app/scripts.json`
- Windows: `%APPDATA%/com.scriptbutler.app/scripts.json`

Hosted script copies are stored in the `scripts/` subdirectory of the same app data directory, with file names made of a timestamp plus the original extension.
