#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/mas"
APP_PATH=""
TEAM_ID=""
APP_CERT=""
INSTALLER_CERT=""
PROVISION_PROFILE=""
APPLE_ID=""
APPLE_APP_PASSWORD=""
PROVIDER_SHORT_NAME=""
SKIP_UPLOAD="false"

usage() {
  cat <<EOF
Usage:
  bash scripts/release_mas.sh [options]

Required:
  --team-id TEAMID
  --app-cert "3rd Party Mac Developer Application: ..."
  --installer-cert "3rd Party Mac Developer Installer: ..."

Optional:
  --provision-profile /abs/path/to/embedded.provisionprofile
  --apple-id you@example.com
  --app-password app-specific-password
  --provider-short-name PROVIDER
  --skip-upload

Notes:
  - Builds app bundle via: npm run tauri build -- --bundles app
  - Signs with entitlements at src-tauri/entitlements/mas*.plist
  - Produces signed pkg at dist/mas/*.pkg
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --team-id) TEAM_ID="${2:-}"; shift 2 ;;
    --app-cert) APP_CERT="${2:-}"; shift 2 ;;
    --installer-cert) INSTALLER_CERT="${2:-}"; shift 2 ;;
    --provision-profile) PROVISION_PROFILE="${2:-}"; shift 2 ;;
    --apple-id) APPLE_ID="${2:-}"; shift 2 ;;
    --app-password) APPLE_APP_PASSWORD="${2:-}"; shift 2 ;;
    --provider-short-name) PROVIDER_SHORT_NAME="${2:-}"; shift 2 ;;
    --skip-upload) SKIP_UPLOAD="true"; shift ;;
    --help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

if [[ -z "$TEAM_ID" || -z "$APP_CERT" || -z "$INSTALLER_CERT" ]]; then
  echo "Missing required args." >&2
  usage
  exit 1
fi

if [[ -n "$PROVISION_PROFILE" && ! -f "$PROVISION_PROFILE" ]]; then
  echo "Provision profile not found: $PROVISION_PROFILE" >&2
  exit 1
fi

if [[ "$SKIP_UPLOAD" != "true" ]]; then
  if [[ -z "$APPLE_ID" || -z "$APPLE_APP_PASSWORD" ]]; then
    echo "Upload enabled but missing --apple-id/--app-password." >&2
    exit 1
  fi
fi

ENTITLEMENTS_APP="$ROOT_DIR/src-tauri/entitlements/mas.plist"
ENTITLEMENTS_INHERIT="$ROOT_DIR/src-tauri/entitlements/mas.inherit.plist"
ICNS_ICON="$ROOT_DIR/src-tauri/icons/icon.icns"
if [[ ! -f "$ENTITLEMENTS_APP" || ! -f "$ENTITLEMENTS_INHERIT" ]]; then
  echo "Missing entitlements files under src-tauri/entitlements." >&2
  exit 1
fi
if [[ ! -f "$ICNS_ICON" ]]; then
  echo "Missing macOS icon: $ICNS_ICON" >&2
  echo "Run: bash scripts/generate_macos_icons.sh /abs/path/to/1024x1024.png" >&2
  exit 1
fi

echo "[mas] Build app bundle"
cd "$ROOT_DIR"
npm run tauri build -- --bundles app

APP_PATH="$(find "$ROOT_DIR/src-tauri/target/release/bundle/macos" -maxdepth 1 -name "*.app" -print | head -n 1)"
if [[ -z "$APP_PATH" ]]; then
  echo "Unable to find .app bundle after build." >&2
  exit 1
fi

echo "[mas] App bundle: $APP_PATH"

if [[ -n "$PROVISION_PROFILE" ]]; then
  echo "[mas] Embed provisioning profile"
  cp "$PROVISION_PROFILE" "$APP_PATH/Contents/embedded.provisionprofile"
fi

echo "[mas] Sign nested binaries/frameworks"
while IFS= read -r target; do
  codesign --force --sign "$APP_CERT" --timestamp --options runtime --entitlements "$ENTITLEMENTS_INHERIT" "$target"
done < <(find "$APP_PATH/Contents" \( -name "*.dylib" -o -name "*.so" -o -name "*.framework" -o -name "*.app" -o -name "*.xpc" \) -print | sort -r)

echo "[mas] Sign main app"
codesign --force --sign "$APP_CERT" --timestamp --options runtime --entitlements "$ENTITLEMENTS_APP" "$APP_PATH"

echo "[mas] Verify app signature"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

mkdir -p "$DIST_DIR"
PKG_PATH="$DIST_DIR/$(basename "$APP_PATH" .app)-mas.pkg"

echo "[mas] Build installer pkg"
productbuild \
  --component "$APP_PATH" /Applications \
  --sign "$INSTALLER_CERT" \
  "$PKG_PATH"

echo "[mas] Verify installer pkg signature"
pkgutil --check-signature "$PKG_PATH"

if [[ "$SKIP_UPLOAD" == "true" ]]; then
  echo "[mas] Skip upload. Package ready: $PKG_PATH"
  exit 0
fi

if ! command -v xcrun >/dev/null 2>&1; then
  echo "xcrun not found, cannot upload." >&2
  exit 1
fi

echo "[mas] Upload via iTMSTransporter"
UPLOAD_CMD=(xcrun iTMSTransporter -m upload -assetFile "$PKG_PATH" -u "$APPLE_ID" -p "$APPLE_APP_PASSWORD")
if [[ -n "$PROVIDER_SHORT_NAME" ]]; then
  UPLOAD_CMD+=(-itc_provider "$PROVIDER_SHORT_NAME")
fi
"${UPLOAD_CMD[@]}"

echo "[mas] Upload complete: $PKG_PATH"
