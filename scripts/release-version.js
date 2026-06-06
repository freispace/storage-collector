#!/usr/bin/env node

import { execSync } from "child_process";
import { readFile, writeFile } from "fs/promises";
import { join } from "path";
import { createInterface } from "readline/promises";
import { stdin as input, stdout as output } from "process";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\/([A-Z]:)/, "$1");
const FILES = {
  packageJson: join(ROOT, "package.json"),
  cargoToml: join(ROOT, "src-tauri", "Cargo.toml"),
  tauriConf: join(ROOT, "src-tauri", "tauri.conf.json"),
};

function run(cmd) {
  console.log(`\n> ${cmd}`);
  execSync(cmd, { cwd: ROOT, stdio: "inherit" });
}

function runText(cmd) {
  return execSync(cmd, { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" }).trim();
}

function localTagExists(tagName) {
  try {
    runText(`git rev-parse -q --verify refs/tags/${tagName}`);
    return true;
  } catch (error) {
    // Exit code 1 means the ref does not exist, which is expected for new tags.
    if (error?.status === 1) {
      return false;
    }
    throw error;
  }
}

function parseSemver(version) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match) {
    throw new Error(`Current version is not strict semver x.y.z: ${version}`);
  }

  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
  };
}

function suggestNextVersion(currentVersion) {
  const parsed = parseSemver(currentVersion);
  return `${parsed.major}.${parsed.minor}.${parsed.patch + 1}`;
}

function isValidSemver(version) {
  return /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version);
}

function updateCargoVersion(cargoToml, nextVersion) {
  const updated = cargoToml.replace(/^(version\s*=\s*")[^"]+("\s*)$/m, `$1${nextVersion}$2`);
  if (updated === cargoToml) {
    throw new Error("Could not update version in src-tauri/Cargo.toml");
  }
  return updated;
}

function updateTauriVersion(tauriConfRaw, nextVersion) {
  const tauriJson = JSON.parse(tauriConfRaw);
  tauriJson.version = nextVersion;
  return `${JSON.stringify(tauriJson, null, 2)}\n`;
}

async function main() {
  const packageJsonRaw = await readFile(FILES.packageJson, "utf8");
  const packageJson = JSON.parse(packageJsonRaw);
  const currentVersion = packageJson.version;
  const suggestedVersion = suggestNextVersion(currentVersion);

  const rl = createInterface({ input, output });

  try {
    console.log(`Current version: ${currentVersion}`);
    const versionInput = await rl.question(`Next version [${suggestedVersion}]: `);
    const nextVersion = (versionInput || suggestedVersion).trim();

    if (!isValidSemver(nextVersion)) {
      throw new Error(`Invalid version: ${nextVersion}. Expected x.y.z`);
    }

    const tagName = `v${nextVersion}`;

    if (localTagExists(tagName)) {
      throw new Error(`Tag ${tagName} already exists locally.`);
    }

    const remoteTags = runText("git ls-remote --tags origin");
    if (remoteTags.includes(`refs/tags/${tagName}`)) {
      throw new Error(`Tag ${tagName} already exists on origin.`);
    }

    console.log("\nPlanned actions:");
    console.log(`- Update version to ${nextVersion} in package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json`);
    console.log(`- Commit with message: chore: release ${tagName}`);
    console.log(`- Create tag: ${tagName}`);
    console.log(`- Push tag: git push origin ${tagName}`);

    const confirm = (await rl.question("Proceed? Type 'yes' to continue: ")).trim().toLowerCase();
    if (confirm !== "yes") {
      console.log("Cancelled.");
      return;
    }

    packageJson.version = nextVersion;
    await writeFile(FILES.packageJson, `${JSON.stringify(packageJson, null, 2)}\n`, "utf8");

    const cargoTomlRaw = await readFile(FILES.cargoToml, "utf8");
    await writeFile(FILES.cargoToml, updateCargoVersion(cargoTomlRaw, nextVersion), "utf8");

    const tauriConfRaw = await readFile(FILES.tauriConf, "utf8");
    await writeFile(FILES.tauriConf, updateTauriVersion(tauriConfRaw, nextVersion), "utf8");

    const filesToCommit = "package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json";
    run(`git add ${filesToCommit}`);
    run(`git commit -m \"chore: release ${tagName}\" -- ${filesToCommit}`);
    run(`git tag ${tagName}`);
    run(`git push origin ${tagName}`);

    console.log(`\nRelease tag ${tagName} pushed successfully.`);
  } finally {
    rl.close();
  }
}

main().catch((error) => {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
});
