#!/usr/bin/env node
"use strict";

// Downloads the dagRobin binary matching this package version from the
// GitHub release, verifies its SHA256 and unpacks it into bin/.

const fs = require("fs");
const os = require("os");
const path = require("path");
const https = require("https");
const crypto = require("crypto");
const { execFileSync } = require("child_process");

const { version } = require("./package.json");
const REPO = "afa7789/dagrobin";
const BIN_DIR = path.join(__dirname, "bin");
const BIN_PATH = path.join(BIN_DIR, "dagRobin");

const TARGETS = {
  "darwin-x64": "dagRobin-macos-amd64",
  "darwin-arm64": "dagRobin-macos-arm64",
  "linux-x64": "dagRobin-linux-amd64",
  "linux-arm64": "dagRobin-linux-arm64",
};

function artifactName() {
  const key = `${process.platform}-${process.arch}`;
  const artifact = TARGETS[key];
  if (!artifact) {
    throw new Error(
      `dagRobin has no prebuilt binary for ${key}. ` +
        `Build from source instead: cargo install --git https://github.com/${REPO}`
    );
  }
  return artifact;
}

function download(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > 5) {
      reject(new Error(`Too many redirects for ${url}`));
      return;
    }
    https
      .get(url, { headers: { "user-agent": "dagrobin-npm-installer" } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume();
          resolve(download(res.headers.location, redirects + 1));
          return;
        }
        if (res.statusCode !== 200) {
          res.resume();
          reject(new Error(`Download failed (HTTP ${res.statusCode}): ${url}`));
          return;
        }
        const chunks = [];
        res.on("data", (c) => chunks.push(c));
        res.on("end", () => resolve(Buffer.concat(chunks)));
        res.on("error", reject);
      })
      .on("error", reject);
  });
}

function assertChecksum(tarball, checksumText, fileName) {
  const expected = checksumText.trim().split(/\s+/)[0];
  const actual = crypto.createHash("sha256").update(tarball).digest("hex");
  if (expected !== actual) {
    throw new Error(
      `Checksum mismatch for ${fileName}: expected ${expected}, got ${actual}`
    );
  }
}

async function main() {
  const artifact = artifactName();
  const base = `https://github.com/${REPO}/releases/download/v${version}`;
  const tarName = `${artifact}.tar.gz`;

  console.log(`dagRobin: downloading ${tarName} (v${version})`);
  const [tarball, checksum] = await Promise.all([
    download(`${base}/${tarName}`),
    download(`${base}/${tarName}.sha256`),
  ]);
  assertChecksum(tarball, checksum.toString("utf8"), tarName);

  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "dagrobin-"));
  const tarPath = path.join(tmp, tarName);
  fs.writeFileSync(tarPath, tarball);
  fs.mkdirSync(BIN_DIR, { recursive: true });
  execFileSync("tar", ["-xzf", tarPath, "-C", BIN_DIR], { stdio: "inherit" });
  fs.rmSync(tmp, { recursive: true, force: true });

  fs.chmodSync(BIN_PATH, 0o755);
  console.log(`dagRobin: installed at ${BIN_PATH}`);
}

main().catch((err) => {
  console.error(`dagRobin install failed: ${err.message}`);
  process.exit(1);
});
