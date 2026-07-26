#!/usr/bin/env node
// Thin pass-through to the binary that install.js downloaded. stdio must be
// inherited or the TUI has no terminal to draw on.
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const bin = path.join(__dirname, process.platform === "win32" ? "git-peek.exe" : "git-peek");
if (!fs.existsSync(bin)) {
  console.error(
    "gitpeek: binary missing - the install step did not run or failed.\n" +
      "Reinstall without --ignore-scripts, or run: node node_modules/git-peek/install.js",
  );
  process.exit(1);
}

const { status, error } = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (error) {
  console.error(`gitpeek: ${error.message}`);
  process.exit(1);
}
process.exit(status ?? 1);
