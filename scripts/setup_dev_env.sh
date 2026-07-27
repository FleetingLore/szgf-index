#!/usr/bin/env bash
set -euo pipefail
# setup_dev_env.sh - check dev environment and optionally install (macOS)

AUTO_INSTALL=0
if [ "${1:-}" = "--install" ]; then
  AUTO_INSTALL=1
fi

OS_TYPE="$(uname -s)"

need=()
check_cmd() { command -v "$1" >/dev/null 2>&1; }

for cmd in git node npm cargo rustup rsync ssh; do
  if ! check_cmd "$cmd"; then
    need+=("$cmd")
  fi
done

if [ ${#need[@]} -eq 0 ]; then
  echo "✅ All required tools are present.";
  exit 0
fi

echo "⚠ Missing tools: ${need[*]}"

if [ "$AUTO_INSTALL" -eq 1 ] && [ "$OS_TYPE" = "Darwin" ]; then
  if ! check_cmd brew; then
    echo "Homebrew not found. Please install Homebrew first: https://brew.sh/"; exit 1
  fi
  echo "Installing missing tools via brew..."
  for pkg in "${need[@]}"; do
    case "$pkg" in
      node|npm)
        brew install node || true
        ;;
      cargo|rustup)
        brew install rustup-init || true
        rustup-init -y || true
        ;;
      rsync)
        brew install rsync || true
        ;;
      ssh)
        brew install openssh || true
        ;;
      git)
        brew install git || true
        ;;
      *)
        echo "No automated installer for $pkg; please install it manually." ;;
    esac
  done
  echo "✅ Installation attempted. Verify tools by re-running this script.";
  exit 0
fi

echo "Please install the following tools and re-run this script:";
for pkg in "${need[@]}"; do
  case "$pkg" in
    node|npm)
      echo "  - Node.js / npm: https://nodejs.org/"
      ;;
    cargo|rustup)
      echo "  - Rust: https://rustup.rs/"
      ;;
    rsync)
      echo "  - rsync (package manager e.g. apt/brew install rsync)"
      ;;
    ssh)
      echo "  - OpenSSH client (ssh)"
      ;;
    git)
      echo "  - git"
      ;;
    *)
      echo "  - $pkg"
      ;;
  esac
done

echo ""
echo "On macOS you can run: scripts/setup_dev_env.sh --install"
echo "On Linux use your distro package manager (apt/yum/pacman) to install the missing tools."
