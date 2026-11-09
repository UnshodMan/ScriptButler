---
name: script-butler
description: "Manage and run local scripts in ScriptButler via the sbctl CLI. Invoke when the user asks to add, host/manage, list, search, run, or remove scripts with ScriptButler."
---

# ScriptButler CLI (sbctl)

Use the `sbctl` command-line tool to manage scripts stored by the ScriptButler desktop app.

## Check availability

- Run `sbctl --version`. If the command is not found, look for it next to the ScriptButler app or ask the user to add it to PATH with `sbctl env add`.
- Run `sbctl path --json` to see the data directory, `scripts.json`, the managed directory, and the executable path.

## Rules

- Always pass `--json` and parse the `{ "ok": true, "data": ... }` envelope.
- Use absolute paths.
- Never edit `scripts.json` directly.
- `remove` only removes the ScriptButler entry. It never deletes the script file or the hosted copy; ask the user before deleting files.
- If a command returns `ambiguous` (exit code 2), rerun with the full path from the error message.
- If `sbctl run --json` returns a non-zero `exitCode`, report the script's stdout/stderr; the sbctl process exits with the same code.

## Hand a generated script to ScriptButler

1. Save the script to an absolute path.
2. `sbctl list --json` to check for duplicates.
3. `sbctl manage <absolute-path> --json` to copy the script into ScriptButler's managed directory.
4. Report `data.name`, `data.path`, `copiedTo`, and `data.effectiveCommand`.

Use `sbctl add <absolute-path> --json` instead when the user wants to keep the script in place (no copy).

After `manage`, both the original entry and the managed copy exist, so referring to the script by bare file name becomes `ambiguous`. Use `data.path` from the `manage` result (or the absolute source path) for later commands.

Use `--command '<template>'` when the default run command is wrong. `{file}` is replaced with the script path; when omitted, the path is appended automatically. Example: `sbctl manage ./task.py --command 'python3 {file}' --json`.

Use `--name`, `--scheme-name`, and `--workdir` to control the display name, default scheme name, and execution directory.

## Run

- `sbctl run <name|path> --json` runs the active scheme and returns `data.exitCode`, `data.stdout`, and `data.stderr`.
- `sbctl run <name|path> --scheme <id|name> --json` runs a specific scheme.
- Output is collected; for long-running scripts prefer the desktop app's live output panel.

## Search

- `sbctl search <keyword> --json` matches file names and execution directories.
- Add `--content` to scan file contents: `sbctl search <keyword> --content --json`.

## Inspect

- `sbctl show <name|path> --json` returns the entry, all schemes, the active scheme, `execDir`, and `effectiveCommand`.
- `sbctl list [--query <keyword>] --json` returns entries in MRU order.

## Remove

- `sbctl remove <name|path> --json` only removes the entry. Files are untouched.

## Errors and exit codes

| Exit code | Meaning |
| --- | --- |
| 0 | success |
| 1 | runtime error (I/O, spawn, manage) |
| 2 | not found, ambiguous reference, or usage error |
| 3 | invalid `scripts.json` |

JSON errors look like `{ "ok": false, "error": { "code": "not_found", "message": "..." } }`.
If the data file is invalid, do not overwrite it; report the problem to the user.

## Install this skill

`sbctl skill` prints this document. `sbctl skill --install <agent-skills-root>` writes `<agent-skills-root>/script-butler/SKILL.md`.
