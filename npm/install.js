#!/usr/bin/env node

const https = require("https");
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

const REPO = "ice-a/spider-cli";
const BINARY = "reptool";

function getPlatform() {
  const platform = process.platform;
  const arch = process.arch;

  const map = {
    "win32-x64": "win32-x64",
    "linux-x64": "linux-x64",
    "linux-arm64": "linux-arm64",
    "darwin-x64": "darwin-x64",
    "darwin-arm64": "darwin-arm64",
  };

  const key = `${platform}-${arch}`;
  return map[key] || null;
}

function getVersion() {
  const pkg = JSON.parse(fs.readFileSync(path.join(__dirname, "package.json"), "utf8"));
  return pkg.version;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const followRedirects = (url) => {
      https.get(url, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          followRedirects(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`HTTP ${res.statusCode}: ${url}`));
          return;
        }
        const file = fs.createWriteStream(dest);
        res.pipe(file);
        file.on("finish", () => {
          file.close();
          resolve();
        });
      }).on("error", reject);
    };
    followRedirects(url);
  });
}

async function main() {
  const platform = getPlatform();
  if (!platform) {
    console.error(`reptool: 不支持的平台 ${process.platform}-${process.arch}`);
    console.error("支持的平台: win32-x64, linux-x64, linux-arm64, darwin-x64, darwin-arm64");
    process.exit(1);
  }

  const version = getVersion();
  const ext = process.platform === "win32" ? ".exe" : "";
  const zipName = `reptool-${platform}.zip`;
  const url = `https://github.com/${REPO}/releases/download/v${version}/${zipName}`;

  const binDir = path.join(__dirname, "bin");
  if (!fs.existsSync(binDir)) fs.mkdirSync(binDir, { recursive: true });

  const binPath = path.join(binDir, BINARY + ext);
  const tmpZip = path.join(binDir, zipName);

  console.log(`reptool: 下载 v${version} (${platform})...`);
  console.log(`  URL: ${url}`);

  try {
    await download(url, tmpZip);

    // 解压
    if (process.platform === "win32") {
      execSync(`powershell -command "Expand-Archive -Path '${tmpZip}' -DestinationPath '${binDir}' -Force"`);
    } else {
      execSync(`unzip -o "${tmpZip}" -d "${binDir}"`);
    }

    // 清理 zip
    fs.unlinkSync(tmpZip);

    // 设置执行权限
    if (process.platform !== "win32") {
      fs.chmodSync(binPath, 0o755);
    }

    console.log(`reptool: 安装成功! ${binPath}`);
  } catch (err) {
    console.error(`reptool: 安装失败 - ${err.message}`);
    console.error(`reptool: 请手动从 https://github.com/${REPO}/releases 下载`);
    process.exit(1);
  }
}

main();