#!/bin/sh
# 已完成双架构 release 构建后，生成可离线安装的通用程序包。
set -eu
[ "$(uname -s)" = Darwin ] || { echo '  err 打包需要 macOS 的 lipo' >&2; exit 1; }
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
PACKAGE_DIR=${1:-"$PROJECT_DIR/../fqdt-macos-universal"}
mkdir -p "$PACKAGE_DIR"
lipo -create "$PROJECT_DIR/target/aarch64-apple-darwin/release/fqdt" \
    "$PROJECT_DIR/target/x86_64-apple-darwin/release/fqdt" -output "$PACKAGE_DIR/fqdt"
cp "$PROJECT_DIR/scripts/install-binary-macos.sh" "$PACKAGE_DIR/install.sh"
cp "$PROJECT_DIR/README.md" "$PACKAGE_DIR/README.md"
(cd "$PACKAGE_DIR" && shasum -a 256 fqdt > SHA256SUMS)
printf '  ok 安装包目录: %s\n' "$PACKAGE_DIR"
