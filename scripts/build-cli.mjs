#!/usr/bin/env node
// Build the sbctl sidecar for the requested (or host) target and place it in
// src-tauri/binaries with the target-triple suffix expected by Tauri.
import { execFileSync } from "node:child_process";
import { chmodSync, copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const exe = process.platform === "win32" ? ".exe" : "";

function hostTriple() {
  try {
    return execFileSync("rustc", ["--print", "host-tuple"], {
      encoding: "utf8",
    }).trim();
  } catch {
    const out = execFileSync("rustc", ["-Vv"], { encoding: "utf8" });
    const match = /host:\s*(\S+)/.exec(out);
    if (!match) throw new Error("Cannot determine the Rust host target triple");
    return match[1];
  }
}

const host = hostTriple();
const requested = (process.env.TAURI_ENV_TARGET_TRIPLE || process.argv[2] || "").trim();
const target = requested || host;

const cargoArgs = [
  "build",
  "--release",
  "-p",
  "scriptbutler-cli",
  "--manifest-path",
  join(root, "src-tauri", "Cargo.toml"),
];
if (target !== host) cargoArgs.push("--target", target);

console.log(`[build-cli] Building sbctl for ${target}...`);
execFileSync("cargo", cargoArgs, { stdio: "inherit" });

const releaseDir =
  target === host
    ? join(root, "src-tauri", "target", "release")
    : join(root, "src-tauri", "target", target, "release");
const source = join(releaseDir, `sbctl${exe}`);
const binDir = join(root, "src-tauri", "binaries");
mkdirSync(binDir, { recursive: true });

const destination = join(binDir, `sbctl-${target}${exe}`);
copyFileSync(source, destination);
if (process.platform !== "win32") chmodSync(destination, 0o755);
console.log(`[build-cli] Sidecar ready: ${destination}`);
