#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

// 修复路径: 从安装目录找到 package.json
const installDir = path.dirname(__filename || __dirname);
const packagePath = path.join(installDir, '..', 'package.json');

let version = '0.1.0';
try {
  const pkg = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
  version = pkg.version;
} catch (e) {
  // 如果找不到 package.json，使用默认版本
}

const platform = process.platform;
const arch = process.arch;

const platformMap = {
  'win32-x64': 'reptool-win-x64.exe',
  'linux-x64': 'reptool-linux-x64',
  'darwin-x64': 'reptool-macos-x64',
  'darwin-arm64': 'reptool-macos-arm64',
};

const binaryName = platformMap[`${platform}-${arch}`];

if (!binaryName) {
  console.log(`reptool: 不支持的平台 ${platform}-${arch}`);
  console.log('请从源码编译: cargo build --release');
  process.exit(0);
}

const binDir = path.join(__dirname, '..', 'bin');
const binPath = path.join(binDir, binaryName);

// 检查是否已有二进制
if (fs.existsSync(binPath)) {
  console.log('reptool: 二进制已存在');
  process.exit(0);
}

// 检查 bin/reptool 是否存在（已经复制好的）
const localBin = path.join(binDir, 'reptool');
const localExe = path.join(binDir, 'reptool.exe');
if (fs.existsSync(localBin) || fs.existsSync(localExe)) {
  console.log('reptool: 二进制已存在');
  process.exit(0);
}

console.log(`reptool: 跳过下载 (本地安装模式)`);
console.log('请确保 bin/ 目录下有 reptool 二进制文件');
