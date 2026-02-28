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

TS="$(date +%Y%m%d-%H%M%S)"
LOG1="$LOG_DIR/tauri-build-$TS-attempt1.log"

if run_once "$LOG1"; then
  echo "[build] success on attempt 1"
  exit 0
fi

if rg -q "bundle_dmg.sh|failed to bundle project" "$LOG1"; then
  echo "[build] dmg bundling failure detected, retrying once..."
  sleep 2
  LOG2="$LOG_DIR/tauri-build-$TS-attempt2.log"
  if run_once "$LOG2"; then
    echo "[build] success on attempt 2"
    exit 0
  fi
fi

echo "[build] failed, check logs under $LOG_DIR"
exit 1
