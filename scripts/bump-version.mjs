#!/usr/bin/env node
// Keep every place that carries the app version in sync:
// package.json / package-lock.json / tauri.conf.json / Cargo.toml / Cargo.lock
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const paths = {
  pkg: join(root, "package.json"),
  pkgLock: join(root, "package-lock.json"),
  tauri: join(root, "src-tauri/tauri.conf.json"),
  cargo: join(root, "src-tauri/Cargo.toml"),
  cargoLock: join(root, "src-tauri/Cargo.lock"),
};

const SEMVER = /^(\d+)\.(\d+)\.(\d+)$/;

function fail(message) {
  console.error(message);
  process.exit(1);
}

function bump(current, arg) {
  const match = SEMVER.exec(current);
  if (!match) fail(`Current version "${current}" in package.json is not x.y.z`);

  const [, major, minor, patch] = match.map(Number);
  switch (arg) {
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "major":
      return `${major + 1}.0.0`;
    default:
      if (SEMVER.test(arg)) return arg;
      fail(
        `Invalid argument "${arg}".\n` +
          "Usage: npm run version:bump -- <patch|minor|major|x.y.z>",
      );
  }
}

function replaceOnce(content, pattern, replacement, label) {
  if (!pattern.test(content)) fail(`Cannot locate ${label}`);
  return content.replace(pattern, replacement);
}

const arg = process.argv[2]?.trim();
if (!arg) fail("Usage: npm run version:bump -- <patch|minor|major|x.y.z>");

const pkg = JSON.parse(readFileSync(paths.pkg, "utf8"));
const current = pkg.version;
const next = bump(current, arg);

pkg.version = next;

const pkgLock = JSON.parse(readFileSync(paths.pkgLock, "utf8"));
pkgLock.version = next;
if (pkgLock.packages?.[""]) pkgLock.packages[""].version = next;

const tauri = JSON.parse(readFileSync(paths.tauri, "utf8"));
tauri.version = next;

const cargo = readFileSync(paths.cargo, "utf8");
const crateName = /\[package\][\s\S]*?\nname\s*=\s*"([^"]+)"/.exec(cargo)?.[1];
if (!crateName) fail("Cannot locate [package] name in src-tauri/Cargo.toml");

const cargoNext = replaceOnce(
  cargo,
  /(\[package\](?:[\s\S]*?\n)version\s*=\s*")[^"]+(")/,
  `$1${next}$2`,
  "[package] version in src-tauri/Cargo.toml",
);

const escapedName = crateName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const cargoLockNext = replaceOnce(
  readFileSync(paths.cargoLock, "utf8"),
  new RegExp(`(\\[\\[package\\]\\]\\nname = "${escapedName}"\\nversion = ")[^"]+(")`),
  `$1${next}$2`,
  `version of "${crateName}" in src-tauri/Cargo.lock`,
);

writeFileSync(paths.pkg, JSON.stringify(pkg, null, 2) + "\n");
writeFileSync(paths.pkgLock, JSON.stringify(pkgLock, null, 2) + "\n");
writeFileSync(paths.tauri, JSON.stringify(tauri, null, 2) + "\n");
writeFileSync(paths.cargo, cargoNext);
writeFileSync(paths.cargoLock, cargoLockNext);

console.log(`Version bumped: ${current} -> ${next}`);
console.log("  package.json");
console.log("  package-lock.json");
console.log("  src-tauri/tauri.conf.json");
console.log("  src-tauri/Cargo.toml");
console.log("  src-tauri/Cargo.lock");
console.log(`\nNext: git tag v${next} && git push origin v${next}`);
