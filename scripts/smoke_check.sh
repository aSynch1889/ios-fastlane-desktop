#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKILL_BOOTSTRAP="${CODEX_HOME:-$HOME/.codex}/skills/ios-fastlane-skill/scripts/bootstrap_fastlane.sh"
IOS_PROJECT=""
RUN_DEV="false"
RUBY_VERSION="3.3.1"

usage() {
  cat <<EOF
Usage:
  bash scripts/smoke_check.sh [options]

Options:
  --ios-project /abs/path   Also run iOS sample checks on a real project
  --run-dev                 When --ios-project is set, run fastlane lane dev
  --ruby-version 3.3.1      Ruby version for iOS sample checks (default: 3.3.1)
  --help                    Show this help

Examples:
  bash scripts/smoke_check.sh
  bash scripts/smoke_check.sh --ios-project /Users/me/project/MyApp
  bash scripts/smoke_check.sh --ios-project /Users/me/project/MyApp --run-dev
EOF
}

step() {
  echo ""
  echo "[smoke] $1"
}

run_cmd() {
  echo "[cmd] $*"
  "$@"
}

run_in_ios_project_with_rvm() {
  local cmd="$1"
  /bin/zsh -lc "source \"$HOME/.rvm/scripts/rvm\" && rvm use \"$RUBY_VERSION\" >/dev/null && cd \"$IOS_PROJECT\" && $cmd"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ios-project)
      IOS_PROJECT="${2:-}"
      shift 2
      ;;
    --run-dev)
      RUN_DEV="true"
      shift
      ;;
    --ruby-version)
      RUBY_VERSION="${2:-}"
      shift 2
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

cd "$ROOT_DIR"

step "Desktop TypeScript build"
run_cmd npm run build

step "Desktop Rust check"
run_cmd bash -lc "cd src-tauri && cargo check"

step "Desktop package build"
run_cmd npm run build:desktop

if [[ -n "$IOS_PROJECT" ]]; then
  if [[ ! -d "$IOS_PROJECT" ]]; then
    echo "iOS project path not found: $IOS_PROJECT" >&2
    exit 1
  fi
  if [[ ! -x "$SKILL_BOOTSTRAP" ]]; then
    echo "Skill bootstrap script not found or not executable: $SKILL_BOOTSTRAP" >&2
    exit 1
  fi

  step "iOS sample: skill dry-run bootstrap"
  run_cmd bash -lc "cd \"$IOS_PROJECT\" && bash \"$SKILL_BOOTSTRAP\" --dry-run"

  step "iOS sample: skill standard bootstrap"
  run_cmd bash -lc "cd \"$IOS_PROJECT\" && bash \"$SKILL_BOOTSTRAP\""

  step "iOS sample: apply Bash 3.2 compatibility patch to doctor script"
  run_cmd bash -lc "cd \"$IOS_PROJECT\" && sed -i '' 's#\${IS_CI,,}#\$(printf '\\''%s'\\'' \"\$IS_CI\" | tr '\\''[:upper:]'\\'' '\\''[:lower:]'\\'')#g' scripts/doctor_fastlane_env.sh"

  step "iOS sample: doctor --fix + validate_config"
  run_in_ios_project_with_rvm "bash scripts/doctor_fastlane_env.sh --project \"\$PWD\" --fix"
  run_in_ios_project_with_rvm "bash scripts/fastlane_run.sh --project \"\$PWD\" --lane validate_config"

  if [[ "$RUN_DEV" == "true" ]]; then
    step "iOS sample: run lane dev"
    run_in_ios_project_with_rvm "FASTLANE_SKIP_UPDATE_CHECK=1 FASTLANE_DISABLE_COLORS=1 CI=1 ENABLE_TESTS=false bundle exec fastlane ios dev"
  else
    echo "[smoke] Skipping lane dev (use --run-dev to enable)"
  fi
fi

echo ""
echo "[smoke] Completed successfully."
