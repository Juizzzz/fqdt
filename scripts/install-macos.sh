#!/bin/sh
# 从本地源码编译安装，不修改 shell 配置，不需要 sudo。
set -eu
if [ "$(uname -s)" != Darwin ]; then
    echo '  err 此脚本仅适用于 macOS' >&2
    exit 1
fi
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
INSTALL_DIR=${FQDT_INSTALL_DIR:-"$HOME/.local/bin"}
CARGO_BIN=${CARGO:-cargo}
if ! command -v "$CARGO_BIN" >/dev/null 2>&1; then
    if [ -x "$HOME/.cargo/bin/cargo" ]; then
        CARGO_BIN="$HOME/.cargo/bin/cargo"
    else
        echo '  err 请先从 https://rust-lang.org/tools/install/ 安装 Rust' >&2
        exit 1
    fi
fi
if ! xcrun --find clang >/dev/null 2>&1; then
    echo '  err 请先运行 xcode-select --install，等待安装完成后重试' >&2
    exit 1
fi
case "$(uname -m)" in
    arm64) TARGET=aarch64-apple-darwin ;;
    x86_64) TARGET=x86_64-apple-darwin ;;
    *) echo '  err 不支持的 Mac 架构' >&2; exit 1 ;;
esac
"$CARGO_BIN" build --manifest-path "$PROJECT_DIR/Cargo.toml" --release --locked \
    --target "$TARGET" --target-dir "$PROJECT_DIR/target"
mkdir -p "$INSTALL_DIR"
INSTALL_TMP=$(mktemp "$INSTALL_DIR/.fqdt.XXXXXX")
trap 'rm -f "$INSTALL_TMP"' EXIT HUP INT TERM
install -m 755 "$PROJECT_DIR/target/$TARGET/release/fqdt" "$INSTALL_TMP"
mv -f "$INSTALL_TMP" "$INSTALL_DIR/fqdt"
printf '  ok 已安装到 %s/fqdt\n' "$INSTALL_DIR"
printf '  下一步: "%s/fqdt" doctor\n' "$INSTALL_DIR"
printf '          "%s/fqdt" init\n' "$INSTALL_DIR"
echo '  如需直接输入 fqdt，请把安装目录加入 PATH。'
