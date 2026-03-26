#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/build-logs"
mkdir -p "$LOG_DIR"

run_once() {
  local log_file="$1"
  echo "[build] log file: $log_file"
  (cd "$ROOT_DIR" && npm run tauri build) 2>&1 | tee "$log_file"
}

run_app_only_once() {
  local log_file="$1"
  echo "[build] app-only fallback log file: $log_file"
  (cd "$ROOT_DIR" && npm run tauri build -- --bundles app) 2>&1 | tee "$log_file"
}

TS="$(date +%Y%m%d-%H%M%S)"
LOG1="$LOG_DIR/tauri-build-$TS-attempt1.log"

if run_once "$LOG1"; then
  echo "[build] success on attempt 1"
  exit 0
fi

if rg -q "failed to read plugin permissions|SCleaner/ios-fastlane-desktop" "$LOG1"; then
  echo "[build] stale tauri target metadata detected, cleaning src-tauri target..."
  (cd "$ROOT_DIR/src-tauri" && cargo clean) >> "$LOG1" 2>&1 || true
fi

if rg -q "bundle_dmg.sh|failed to bundle project|failed to read plugin permissions" "$LOG1"; then
  echo "[build] transient packaging/build failure detected, retrying once..."
  sleep 2
  LOG2="$LOG_DIR/tauri-build-$TS-attempt2.log"
  if run_once "$LOG2"; then
    echo "[build] success on attempt 2"
    exit 0
  fi

  if rg -q "bundle_dmg.sh|failed to bundle project" "$LOG2"; then
    echo "[build] dmg packaging remains unstable, trying app-only fallback..."
    sleep 1
    LOG3="$LOG_DIR/tauri-build-$TS-app-only.log"
    if run_app_only_once "$LOG3"; then
      echo "[build] success with app-only fallback (dmg skipped)"
      exit 0
    fi
  fi
fi

echo "[build] failed, check logs under $LOG_DIR"
exit 1
