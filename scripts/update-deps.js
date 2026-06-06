#!/usr/bin/env node
// update-deps.js — updates pnpm and Rust (Cargo) dependencies

import { execSync } from "child_process";
import { existsSync } from "fs";
import { join } from "path";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1");
const TAURI_DIR = join(ROOT, "src-tauri");

function run(cmd, cwd = ROOT, env = undefined) {
  console.log(`\n> ${cmd}`);
  execSync(cmd, {
    cwd,
    stdio: "inherit",
    env: env ? { ...process.env, ...env } : process.env,
  });
}

function checkTool(cmd) {
  try {
    execSync(`${cmd} --version`, { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

// ── pnpm ──────────────────────────────────────────────────────────────────────
console.log("=== Updating pnpm dependencies ===");

if (!checkTool("pnpm")) {
  console.error("ERROR: pnpm not found. Install it with: npm install -g pnpm");
  process.exit(1);
}

run("pnpm update --latest");

// ── Cargo ─────────────────────────────────────────────────────────────────────
console.log("\n=== Updating Rust (Cargo) dependencies ===");

if (!checkTool("cargo")) {
  console.error("ERROR: cargo not found. Is Rust installed?");
  process.exit(1);
}

if (!existsSync(TAURI_DIR)) {
  console.error(`ERROR: src-tauri directory not found at ${TAURI_DIR}`);
  process.exit(1);
}

// cargo-edit provides `cargo upgrade` to bump versions in Cargo.toml
if (checkTool("cargo upgrade")) {
  run("cargo upgrade --incompatible allow", TAURI_DIR);
} else {
  console.log(
    "INFO: cargo-edit not installed — running `cargo update` (patch/minor only).\n" +
    "      To also bump major versions, install cargo-edit: cargo install cargo-edit"
  );
}

run("cargo update", TAURI_DIR);

// Regenerate sqlx offline metadata after dependency changes
console.log("\n=== Regenerating sqlx offline metadata ===");
const sqlxAvailable = (() => {
  try {
    execSync("cargo sqlx --version", { cwd: TAURI_DIR, stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
})();

if (sqlxAvailable) {
  // sqlx prepare needs a database URL at build time; use a local SQLite file.
  const sqlxPrepareDbUrl = "sqlite://.sqlx-prepare.db?mode=rwc";
  run("cargo sqlx prepare", TAURI_DIR, { DATABASE_URL: sqlxPrepareDbUrl });
  console.log("INFO: Remember to commit the updated .sqlx/ directory.");
} else {
  console.log(
    "INFO: cargo-sqlx CLI not installed — skipping sqlx prepare.\n" +
    "      Install with: cargo install sqlx-cli --no-default-features --features rustls,sqlite"
  );
}

console.log("\nDone. Review any version bumps and run `pnpm build` to verify.");
