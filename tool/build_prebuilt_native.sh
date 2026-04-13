#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RUST_DIR="$ROOT_DIR/native/rust"
PREBUILT_DIR="$ROOT_DIR/native/prebuilt"
HOST_OS="$(uname -s)"
HOST_ARCH="$(uname -m)"

run_cargo() {
  (
    cd "$RUST_DIR"
    cargo "$@"
  )
}

current_desktop_os() {
  case "$HOST_OS" in
    Darwin) echo "macos" ;;
    Linux) echo "linux" ;;
    MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
    *)
      echo "Unsupported host OS: $HOST_OS" >&2
      exit 1
      ;;
  esac
}

current_desktop_arch() {
  case "$HOST_ARCH" in
    arm64|aarch64) echo "arm64" ;;
    x86_64|amd64) echo "x64" ;;
    *)
      echo "Unsupported host architecture: $HOST_ARCH" >&2
      exit 1
      ;;
  esac
}

desktop_rust_target() {
  local os="$1"
  local arch="$2"

  case "$os:$arch" in
    macos:arm64) echo "aarch64-apple-darwin" ;;
    macos:x64) echo "x86_64-apple-darwin" ;;
    linux:arm64) echo "aarch64-unknown-linux-gnu" ;;
    linux:x64) echo "x86_64-unknown-linux-gnu" ;;
    windows:arm64) echo "aarch64-pc-windows-msvc" ;;
    windows:x64) echo "x86_64-pc-windows-msvc" ;;
    *)
      echo "Unsupported desktop target: $os ($arch)" >&2
      exit 1
      ;;
  esac
}

desktop_artifact_name() {
  local os="$1"

  case "$os" in
    macos) echo "libimage_native.dylib" ;;
    linux) echo "libimage_native.so" ;;
    windows) echo "image_native.dll" ;;
    *)
      echo "Unsupported desktop OS: $os" >&2
      exit 1
      ;;
  esac
}

default_target() {
  case "$(current_desktop_os)" in
    macos|linux|windows) current_desktop_os ;;
    *)
      echo "Unsupported default host target." >&2
      exit 1
      ;;
  esac
}

TARGET="${1:-$(default_target)}"

build_android() {
  local abi="$1"
  local out_dir="$PREBUILT_DIR/android"
  mkdir -p "$out_dir"
  run_cargo ndk -t "$abi" -o "$out_dir" build --release
}

build_ios() {
  local device_target="aarch64-apple-ios"
  local sim_target="aarch64-apple-ios-sim"
  local sim_x64_target="x86_64-apple-ios"
  local build_root="$PREBUILT_DIR/ios/build"
  local headers_dir="$build_root/Headers"
  local out_device_dir="$PREBUILT_DIR/ios/iphoneos"
  local out_sim_arm64_dir="$PREBUILT_DIR/ios/iphonesimulator-arm64"
  local out_sim_x64_dir="$PREBUILT_DIR/ios/iphonesimulator-x86_64"
  local device_lib="$build_root/iphoneos/libimage_native.dylib"
  local sim_arm64_lib="$build_root/iphonesimulator/libimage_native.arm64.dylib"
  local sim_x64_lib="$build_root/iphonesimulator/libimage_native.x86_64.dylib"

  run_cargo build --release --target "$device_target"
  run_cargo build --release --target "$sim_target"
  run_cargo build --release --target "$sim_x64_target"

  rm -rf "$build_root"
  rm -rf \
    "$PREBUILT_DIR/ios/ImageNative.xcframework" \
    "$PREBUILT_DIR/ios/iphonesimulator" \
    "$out_device_dir" \
    "$out_sim_arm64_dir" \
    "$out_sim_x64_dir"
  mkdir -p \
    "$headers_dir" \
    "$(dirname "$device_lib")" \
    "$(dirname "$sim_arm64_lib")" \
    "$out_device_dir" \
    "$out_sim_arm64_dir" \
    "$out_sim_x64_dir"

  cp "$ROOT_DIR/native/include/image_ffi.h" "$headers_dir/"
  cp "$RUST_DIR/target/$device_target/release/libimage_native.dylib" "$device_lib"
  cp "$RUST_DIR/target/$sim_target/release/libimage_native.dylib" "$sim_arm64_lib"
  cp "$RUST_DIR/target/$sim_x64_target/release/libimage_native.dylib" "$sim_x64_lib"

  install_name_tool -id "@rpath/libimage_native.dylib" "$device_lib"
  install_name_tool -id "@rpath/libimage_native.dylib" "$sim_arm64_lib"
  install_name_tool -id "@rpath/libimage_native.dylib" "$sim_x64_lib"
  cp "$device_lib" "$out_device_dir/libimage_native.dylib"
  cp "$sim_arm64_lib" "$out_sim_arm64_dir/libimage_native.dylib"
  cp "$sim_x64_lib" "$out_sim_x64_dir/libimage_native.dylib"
}

build_desktop() {
  local os="$1"
  local expected_os
  expected_os="$(current_desktop_os)"
  if [[ "$os" != "$expected_os" ]]; then
    echo "Cannot build $os prebuilts on $expected_os host." >&2
    exit 1
  fi

  local arch
  arch="$(current_desktop_arch)"
  local rust_target
  rust_target="$(desktop_rust_target "$os" "$arch")"
  local artifact_name
  artifact_name="$(desktop_artifact_name "$os")"
  local out_dir="$PREBUILT_DIR/$os/$arch"

  mkdir -p "$out_dir"
  run_cargo build --release --target "$rust_target"
  cp "$RUST_DIR/target/$rust_target/release/$artifact_name" "$out_dir/$artifact_name"
}

case "$TARGET" in
  all|android|ios|macos|linux|windows) ;;
  *)
    echo "Usage: $0 [android|ios|macos|linux|windows|all]" >&2
    exit 1
    ;;
esac

if [[ "$TARGET" == "android" || "$TARGET" == "all" ]]; then
  build_android arm64-v8a
  build_android armeabi-v7a
  build_android x86_64
fi

if [[ "$TARGET" == "ios" || "$TARGET" == "all" ]]; then
  build_ios
fi

if [[ "$TARGET" == "macos" ]]; then
  build_desktop macos
fi

if [[ "$TARGET" == "linux" ]]; then
  build_desktop linux
fi

if [[ "$TARGET" == "windows" ]]; then
  build_desktop windows
fi

if [[ "$TARGET" == "all" ]]; then
  build_desktop "$(current_desktop_os)"
fi
