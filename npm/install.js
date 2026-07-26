// Downloads the prebuilt git-peek binary for this platform from the matching
// GitHub Release and verifies it against the published .sha256 before use.
const { createHash } = require("node:crypto");
const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const TARGETS = {
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-musl",
  "linux-arm64": "aarch64-unknown-linux-musl",
  "win32-x64": "x86_64-pc-windows-msvc",
};

const { version } = require("./package.json");
const platform = `${process.platform}-${process.arch}`;
const target = TARGETS[platform];
// Overridable so the download path can be tested without a live release.
const base =
  process.env.GITPEEK_RELEASE_BASE ||
  `https://github.com/tanishq-khandelwal/gitpeek/releases/download/v${version}`;

async function fetchBuffer(url) {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`);
  return Buffer.from(await res.arrayBuffer());
}

async function main() {
  if (!target) {
    throw new Error(
      `no prebuilt binary for ${platform}. Build from source instead: ` +
        `cargo install gitpeek`,
    );
  }

  const archive = `gitpeek-${target}.tar.gz`;
  const [tgz, checksum] = await Promise.all([
    fetchBuffer(`${base}/${archive}`),
    fetchBuffer(`${base}/${archive}.sha256`),
  ]);

  const expected = checksum.toString("utf8").trim().split(/\s+/)[0];
  const actual = createHash("sha256").update(tgz).digest("hex");
  if (!expected || expected !== actual) {
    throw new Error(`checksum mismatch for ${archive}: expected ${expected}, got ${actual}`);
  }

  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });
  const tmp = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "gitpeek-")), archive);
  fs.writeFileSync(tmp, tgz);
  try {
    // `tar` ships with macOS, Linux, and Windows 10 1803+ - no npm dependency needed.
    execFileSync("tar", ["xzf", tmp, "-C", binDir], { stdio: "inherit" });
  } finally {
    fs.rmSync(path.dirname(tmp), { recursive: true, force: true });
  }

  const bin = path.join(binDir, process.platform === "win32" ? "git-peek.exe" : "git-peek");
  if (!fs.existsSync(bin)) throw new Error(`archive did not contain ${path.basename(bin)}`);
  fs.chmodSync(bin, 0o755);
  console.log(`gitpeek ${version}: installed ${target}`);
}

main().catch((err) => {
  console.error(`gitpeek install failed: ${err.message}`);
  process.exit(1);
});
