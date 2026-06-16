#!/bin/bash
# Reptool 发布脚本
# 用法: bash publish.sh [version]

set -e

VERSION=${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}
echo "=== 发布 Reptool v${VERSION} ==="

# 1. 构建 release
echo "[1/5] 构建 release..."
cargo build --release

# 2. 复制二进制到 npm 目录
echo "[2/5] 准备 npm 包..."
cp target/release/reptool npm/bin/reptool 2>/dev/null || true
cp target/release/reptool.exe npm/bin/reptool.exe 2>/dev/null || true
chmod +x npm/bin/reptool 2>/dev/null || true

# 3. 更新 npm 版本
echo "[3/5] 更新版本号..."
cd npm
npm version "$VERSION" --no-git-tag-version 2>/dev/null || true
cd ..

# 4. 打包 Windows 发布包
echo "[4/5] 打包发布包..."
rm -f reptool-win-x64.zip
if [ -f target/release/reptool.exe ]; then
    zip reptool-win-x64.zip target/release/reptool.exe
    echo "  Windows: reptool-win-x64.zip ($(du -h reptool-win-x64.zip | cut -f1))"
fi

# 5. 发布 npm
echo "[5/5] 发布到 npm..."
cd npm
npm publish
cd ..

echo ""
echo "=== 发布完成 ==="
echo "npm: https://www.npmjs.com/package/reptool"
echo "用户安装: npm install -g reptool@${VERSION}"
