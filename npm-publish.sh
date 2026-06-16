#!/bin/bash
# Reptool npm 发布脚本
# 用法: bash npm-publish.sh [version]

set -e

VERSION=${1:-$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')}
echo "=== 发布 Reptool npm 包 v${VERSION} ==="

# 1. 构建当前平台
echo "[1/6] 构建当前平台..."
cargo build --release

# 2. 获取当前平台信息
PLATFORM=$(node -e "console.log(process.platform + '-' + process.arch)")
echo "当前平台: $PLATFORM"

# 3. 复制二进制到对应的平台包
echo "[2/6] 准备平台包..."
case "$PLATFORM" in
  win32-x64)
    mkdir -p npm/win32-x64/bin
    cp target/release/reptool.exe npm/win32-x64/bin/
    ;;
  linux-x64)
    mkdir -p npm/linux-x64/bin
    cp target/release/reptool npm/linux-x64/bin/
    chmod +x npm/linux-x64/bin/reptool
    ;;
  linux-arm64)
    mkdir -p npm/linux-arm64/bin
    cp target/release/reptool npm/linux-arm64/bin/
    chmod +x npm/linux-arm64/bin/reptool
    ;;
  darwin-x64)
    mkdir -p npm/darwin-x64/bin
    cp target/release/reptool npm/darwin-x64/bin/
    chmod +x npm/darwin-x64/bin/reptool
    ;;
  darwin-arm64)
    mkdir -p npm/darwin-arm64/bin
    cp target/release/reptool npm/darwin-arm64/bin/
    chmod +x npm/darwin-arm64/bin/reptool
    ;;
  *)
    echo "不支持的平台: $PLATFORM"
    exit 1
    ;;
esac

# 4. 更新版本号
echo "[3/6] 更新版本号..."

# 5. 更新平台包版本号
echo "[4/6] 更新平台包版本号..."
for platform in win32-x64 linux-x64 linux-arm64 darwin-x64 darwin-arm64; do
  if [ -d "npm/$platform" ]; then
    cd "npm/$platform"
    npm version "$VERSION" --no-git-tag-version 2>/dev/null || true
    cd ../..
  fi
done

# 6. 更新主包版本号和 optionalDependencies
echo "[5/6] 更新主包..."
cd npm/reptool
npm version "$VERSION" --no-git-tag-version 2>/dev/null || true
node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
for (const dep in pkg.optionalDependencies) {
  pkg.optionalDependencies[dep] = '$VERSION';
}
fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"
cd ../..

# 7. 发布
echo "[6/6] 发布到 npm..."
cd npm

# 先发布平台包
for platform in win32-x64 linux-x64 linux-arm64 darwin-x64 darwin-arm64; do
  if [ -d "$platform" ]; then
    echo "发布 reptool-${platform}@${VERSION}..."
    cd "$platform"
    npm publish --access public
    cd ..
  fi
done

# 发布主包
echo "发布 reptool@${VERSION}..."
cd reptool
npm publish --access public
cd ..

cd ..

echo ""
echo "=== 发布完成 ==="
echo "npm: https://www.npmjs.com/package/reptool"
echo "用户安装: npm install -g reptool@${VERSION}"