#!/usr/bin/env bash
# Builds the release GDExtension library for Android and copies it into the Godot project.
# Usage: ANDROID_NDK_HOME=<ndk> scripts/build-android.sh [arm64|x86_64]...   (default: arm64)
# arm64 is the target device ABI; x86_64 exists only so the x86_64 Android emulator can run the same code.
set -euo pipefail
: "${ANDROID_NDK_HOME:?set ANDROID_NDK_HOME to an NDK (r27+; 16 KB page alignment is default since r28)}"
API=24
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) HOST=windows-x86_64; EXT=.cmd ;;
  Darwin) HOST=darwin-x86_64; EXT= ;;
  *) HOST=linux-x86_64; EXT= ;;
esac
BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST/bin"
cd "$(dirname "$0")/.."
ABIS=("${@:-arm64}")
for abi in "${ABIS[@]}"; do
  case "$abi" in
    arm64)  TRIPLE=aarch64-linux-android; CLANG="$BIN/aarch64-linux-android$API-clang$EXT"; OUT=arm64 ;;
    x86_64) TRIPLE=x86_64-linux-android;  CLANG="$BIN/x86_64-linux-android$API-clang$EXT";  OUT=x86_64 ;;
    *) echo "unknown abi $abi" >&2; exit 2 ;;
  esac
  UP=$(echo "$TRIPLE" | tr 'a-z-' 'A-Z_')
  export "CARGO_TARGET_${UP}_LINKER=$CLANG"
  cargo build --release -p cube-godot-adapter --target "$TRIPLE"
  mkdir -p "godot-client/bin/android/$OUT"
  cp "target/$TRIPLE/release/libcube_godot_adapter.so" "godot-client/bin/android/$OUT/"
  echo "built godot-client/bin/android/$OUT/libcube_godot_adapter.so"
done
