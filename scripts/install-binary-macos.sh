#!/bin/sh
# 此文件随 Mac 通用程序一起分发，复制为安装包中的 install.sh。
set -eu
[ "$(uname -s)" = Darwin ] || { echo '  err 此安装包仅适用于 macOS' >&2; exit 1; }
SOURCE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
INSTALL_DIR=${FQDT_INSTALL_DIR:-"$HOME/.local/bin"}
case "$(uname -m)" in
    arm64) ARCH=arm64 ;;
    x86_64) ARCH=x86_64 ;;
    *) echo '  err 不支持的架构' >&2; exit 1 ;;
esac
(cd "$SOURCE_DIR" && shasum -a 256 -c SHA256SUMS)
DESCRIPTION=$(file -b "$SOURCE_DIR/fqdt")
case "$DESCRIPTION" in
    *Mach-O*"$ARCH"*) ;;
    *) echo '  err 安装包格式或架构不匹配' >&2; exit 1 ;;
esac
mkdir -p "$INSTALL_DIR"
INSTALL_TMP=$(mktemp "$INSTALL_DIR/.fqdt.XXXXXX")
trap 'rm -f "$INSTALL_TMP"' EXIT HUP INT TERM
install -m 755 "$SOURCE_DIR/fqdt" "$INSTALL_TMP"
mv -f "$INSTALL_TMP" "$INSTALL_DIR/fqdt"
printf '  ok 已安装到 %s/fqdt\n' "$INSTALL_DIR"
printf '  下一步: "%s/fqdt" doctor\n' "$INSTALL_DIR"
printf '          "%s/fqdt" init\n' "$INSTALL_DIR"
