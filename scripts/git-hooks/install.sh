#!/usr/bin/env bash
set -euo pipefail
HOOK_DIR=".git/hooks"
mkdir -p "$HOOK_DIR"
cp scripts/git-hooks/pre-push "$HOOK_DIR/pre-push"
chmod +x "$HOOK_DIR/pre-push"
echo "Installed pre-push hook. To remove, delete $HOOK_DIR/pre-push"
