#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_PNG="${1:-$ROOT_DIR/src-tauri/icons/icon.png}"
ICONSET_DIR="$ROOT_DIR/src-tauri/icons/AppIcon.iconset"
OUT_ICNS="$ROOT_DIR/src-tauri/icons/icon.icns"

if [[ ! -f "$SRC_PNG" ]]; then
  echo "Source icon not found: $SRC_PNG" >&2
  exit 1
fi

if ! command -v sips >/dev/null 2>&1 || ! command -v iconutil >/dev/null 2>&1; then
  echo "Missing required tools: sips/iconutil" >&2
  exit 1
fi

SRC_WIDTH="$(sips -g pixelWidth "$SRC_PNG" | awk '/pixelWidth:/ {print $2}')"
SRC_HEIGHT="$(sips -g pixelHeight "$SRC_PNG" | awk '/pixelHeight:/ {print $2}')"
if [[ -z "$SRC_WIDTH" || -z "$SRC_HEIGHT" ]]; then
  echo "Unable to read source icon size: $SRC_PNG" >&2
  exit 1
fi
if [[ "$SRC_WIDTH" -lt 1024 || "$SRC_HEIGHT" -lt 1024 ]]; then
  echo "Source icon must be at least 1024x1024 for valid .icns generation." >&2
  echo "Current size: ${SRC_WIDTH}x${SRC_HEIGHT} ($SRC_PNG)" >&2
  exit 1
fi

rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

render() {
  local size="$1"
  local out="$2"
  sips -z "$size" "$size" "$SRC_PNG" --out "$ICONSET_DIR/$out" >/dev/null
}

render 16 icon_16x16.png
render 32 icon_16x16@2x.png
render 32 icon_32x32.png
render 64 icon_32x32@2x.png
render 128 icon_128x128.png
render 256 icon_128x128@2x.png
render 256 icon_256x256.png
render 512 icon_256x256@2x.png
render 512 icon_512x512.png
render 1024 icon_512x512@2x.png

iconutil -c icns "$ICONSET_DIR" -o "$OUT_ICNS"
echo "Generated: $OUT_ICNS"
