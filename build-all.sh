#!/bin/bash
# Reptool 全平台构建脚本
# 用法: bash build-all.sh

set -e

echo "=== Reptool 全平台构建 ==="

# 清理旧产物
rm -rf release/
mkdir -p release/

# 获取版本号
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
echo "版本: $VERSION"

# 构建当前平台
echo ""
echo "--- 构建当前平台 ---"
cargo build --release
cp target/release/reptool release/reptool-$(uname -s)-$(uname -m)
echo "完成: release/reptool-$(uname -s)-$(uname -m)"

# 尝试交叉编译 Linux
echo ""
echo "--- 交叉编译 Linux x64 ---"
if rustup target list --installed | grep -q x86_64-unknown-linux-gnu; then
    cargo build --release --target x86_64-unknown-linux-gnu 2>/dev/null && \
        cp target/x86_64-unknown-linux-gnu/release/reptool release/reptool-linux-x64 || \
        echo "跳过: 需要 Linux 交叉编译工具链"
fi

# 尝试交叉编译 macOS
echo ""
echo "--- 交叉编译 macOS x64 ---"
if rustup target list --installed | grep -q x86_64-apple-darwin; then
    cargo build --release --target x86_64-apple-darwin 2>/dev/null && \
        cp target/x86_64-apple-darwin/release/reptool release/reptool-macos-x64 || \
        echo "跳过: 需要 macOS 交叉编译工具链"
fi

echo ""
echo "--- 交叉编译 macOS ARM64 ---"
if rustup target list --installed | grep -q aarch64-apple-darwin; then
    cargo build --release --target aarch64-apple-darwin 2>/dev/null && \
        cp target/aarch64-apple-darwin/release/reptool release/reptool-macos-arm64 || \
        echo "跳过: 需要 macOS ARM64 交叉编译工具链"
fi

# 打包
echo ""
echo "--- 打包 ---"
cd release/
for f in reptool-*; do
    if [ -f "$f" ]; then
        zip "${f}.zip" "$f"
        echo "打包: ${f}.zip ($(du -h "${f}.zip" | cut -f1))"
    fi
done
cd ..

echo ""
echo "=== 构建完成 ==="
echo "产物目录: release/"
ls -la release/
