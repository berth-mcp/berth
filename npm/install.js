#!/usr/bin/env node
"use strict";

const https = require("https");
const fs = require("fs");
const path = require("path");
const { version } = require("./package.json");

const REPO = "berth-mcp/berth";

function getPlatformAsset() {
  const platform = process.platform;
  const ext = platform === "win32" ? ".exe" : "";
  const suffix =
    platform === "linux"
      ? "linux"
      : platform === "darwin"
        ? "macos"
        : platform === "win32"
          ? "windows"
          : null;

  if (!suffix) {
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
  }

  const tag = `v${version}`;
  const name = `berth-${tag}-${suffix}${ext}`;
  const url = `https://github.com/${REPO}/releases/download/${tag}/${name}`;
  const binName = `berth${ext}`;
  return { url, binName };
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const request = (u) => {
      https
        .get(u, (res) => {
          if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
            request(res.headers.location);
            return;
          }
          if (res.statusCode !== 200) {
            reject(new Error(`Download failed: HTTP ${res.statusCode} from ${u}`));
            return;
          }
          const file = fs.createWriteStream(dest);
          res.pipe(file);
          file.on("finish", () => {
            file.close();
            resolve();
          });
        })
        .on("error", reject);
    };
    request(url);
  });
}

async function main() {
  const { url, binName } = getPlatformAsset();
  const binDir = path.join(__dirname, "bin");
  const dest = path.join(binDir, binName);

  fs.mkdirSync(binDir, { recursive: true });

  console.log(`Downloading berth from ${url}`);
  await download(url, dest);
  fs.chmodSync(dest, 0o755);
  console.log(`Installed berth to ${dest}`);
}

main().catch((err) => {
  console.error(`Failed to install berth: ${err.message}`);
  process.exit(1);
});
