#!/usr/bin/env node
"use strict";

// Thin shim: forwards every argument to the native dagRobin binary that
// install.js placed next to this file.

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const binary = path.join(__dirname, "dagRobin");

if (!fs.existsSync(binary)) {
  console.error(
    "dagRobin binary not found. Re-run the install step:\n" +
      "  npm rebuild dagrobin\n" +
      "or install from source: cargo install --git https://github.com/afa7789/dagrobin"
  );
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(`dagRobin: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
