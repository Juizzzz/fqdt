#!/bin/sh
# 下载本分支发布流程生成的原生程序；可指定 fork 和 tag，不执行下载内容。
set -eu
REPO=${FQDT_REPO:-addallno/fqdt}
VERSION=${FQDT_VERSION:-v0.6.0}
DEST=${1:-./fqdt}
case "$(uname -s):$(uname -m)" in
    Darwin:arm64) ASSET=fqdt-aarch64-apple-darwin; ARCH=arm64; KIND=Mach-O ;;
    Darwin:x86_64) ASSET=fqdt-x86_64-apple-darwin; ARCH=x86_64; KIND=Mach-O ;;
    Linux:aarch64|Linux:arm64) ASSET=fqdt-aarch64-linux; ARCH=aarch64; KIND=ELF ;;
    *) echo '  err 不支持的平台，请从源码编译' >&2; exit 1 ;;
esac
for TOOL in curl file; do
    command -v "$TOOL" >/dev/null 2>&1 || { echo "  err 缺少 $TOOL" >&2; exit 1; }
done
BASE="https://github.com/$REPO/releases/download/$VERSION"
DEST_DIR=$(dirname -- "$DEST")
mkdir -p "$DEST_DIR"
DOWNLOAD_DIR=$(mktemp -d "$DEST_DIR/.fqdt-download.XXXXXX")
trap 'rm -rf "$DOWNLOAD_DIR"' EXIT HUP INT TERM
if ! curl -fL --connect-timeout 15 --max-time 180 "$BASE/$ASSET" -o "$DOWNLOAD_DIR/$ASSET"; then
    echo '  err 下载失败；请确认此仓库/tag 已发布 macOS 适配版，或运行 scripts/install-macos.sh' >&2
    exit 1
fi
curl -fL --connect-timeout 15 --max-time 30 "$BASE/$ASSET.sha256" -o "$DOWNLOAD_DIR/$ASSET.sha256"
# 校验文件仅当作文本读取，不执行。校验本次下载的文件，不读取远端指定的路径。
EXPECTED=$(awk 'NR == 1 {print $1}' "$DOWNLOAD_DIR/$ASSET.sha256")
if command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "$DOWNLOAD_DIR/$ASSET" | awk '{print $1}')
else
    ACTUAL=$(sha256sum "$DOWNLOAD_DIR/$ASSET" | awk '{print $1}')
fi
[ "$EXPECTED" = "$ACTUAL" ] || { echo '  err SHA-256 校验失败' >&2; exit 1; }
DESCRIPTION=$(file -b "$DOWNLOAD_DIR/$ASSET")
case "$DESCRIPTION" in
    *"$KIND"*"$ARCH"*) ;;
    *) echo "  err 程序格式或架构不匹配: $DESCRIPTION" >&2; exit 1 ;;
esac
chmod +x "$DOWNLOAD_DIR/$ASSET"
mv -f "$DOWNLOAD_DIR/$ASSET" "$DEST"
printf '  ok 已下载到 %s\n' "$DEST"
