#!/bin/bash
# Build the spool CLI binary and place it where Tauri's externalBin expects it.
# Usage: ./scripts/build-cli.sh [--target <triple>] [--release]

set -euo pipefail

TARGET=""
PROFILE="debug"

while [[ $# -gt 0 ]]; do
  case $1 in
    --target) TARGET="$2"; shift 2 ;;
    --release) PROFILE="release"; shift ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# Determine target triple
if [ -z "$TARGET" ]; then
  TARGET=$(rustc -vV | grep 'host:' | awk '{print $2}')
fi

# Build CLI
CARGO_ARGS="-p spool-cli"
if [ "$PROFILE" = "release" ]; then
  CARGO_ARGS="$CARGO_ARGS --release"
fi
if [ "$TARGET" != "$(rustc -vV | grep 'host:' | awk '{print $2}')" ]; then
  CARGO_ARGS="$CARGO_ARGS --target $TARGET"
fi

echo "Building spool-cli for $TARGET ($PROFILE)..."
cargo build $CARGO_ARGS

# Determine source path
if [ "$TARGET" != "$(rustc -vV | grep 'host:' | awk '{print $2}')" ]; then
  SRC="target/$TARGET/$PROFILE/spool"
else
  SRC="target/$PROFILE/spool"
fi

# Windows: add .exe suffix
EXT=""
case "$TARGET" in
  *-windows-*) EXT=".exe" ;;
esac

# Copy to Tauri binaries dir
DEST_DIR="src-tauri/binaries"
mkdir -p "$DEST_DIR"
cp "${SRC}${EXT}" "${DEST_DIR}/spool-${TARGET}${EXT}"

echo "Installed: ${DEST_DIR}/spool-${TARGET}${EXT}"
