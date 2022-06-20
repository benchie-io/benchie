#!/bin/sh

set -e

if ! command -v unzip >/dev/null; then
  echo "Error: unzip is required to install benchie." 1>&2
  exit 1
fi

if [ "$OS" = "Windows_NT" ]; then
  target="x86_64-pc-windows-msvc"
else
  case $(uname -sm) in
  "Darwin x86_64") target="apple-darwin" ;;
  "Darwin arm64") target="apple-darwin" ;;
  *) target="x86_64-unknown-linux-musl" ;;
  esac
fi

if [ $# -eq 0 ]; then
  benchie_uri="https://github.com/benchie-io/benchie/releases/latest/download/benchie-${target}.zip"
else
  benchie_uri="https://github.com/benchie-io/benchie/releases/download/${1}/benchie-${target}.zip"
fi

benchie_install="${BENCHIE_INSTALL:-$HOME/.benchie}"
bin_dir="$benchie_install/bin"
exe="$bin_dir/benchie"

if [ ! -d "$bin_dir" ]; then
  mkdir -p "$bin_dir"
fi

curl --fail --location --progress-bar --output "$exe.zip" "$benchie_uri"
unzip -d "$bin_dir" -o "$exe.zip"
chmod +x "$exe"
rm "$exe.zip"

echo "benchie was installed successfully to $exe"
if command -v benchie >/dev/null; then
  echo "Run 'benchie --help' to get started"
else
  case $SHELL in
  /bin/zsh) shell_profile=".zshrc" ;;
  *) shell_profile=".bash_profile" ;;
  esac
  echo "Manually add the directory to your \$HOME/$shell_profile (or similar)"
  echo "  export BENCHIE_INSTALL=\"$benchie_install\""
  echo "  export PATH=\"\$BENCHIE_INSTALL/bin:\$PATH\""
  echo "Run '$exe --help' to get started"
fi