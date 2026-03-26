# iOS Fastlane Desktop (Prototype)

A macOS desktop prototype built with Tauri + React + TypeScript for visual fastlane configuration and lane execution.

## Why this exists

This project turns common iOS fastlane workflows into a GUI:

- Detect project basics (`xcworkspace` / `xcodeproj`)
- Configure signing, distribution and quality flags
- Generate fastlane env files
- Run lanes and inspect logs in-app

## Current scope

- Project scanner command (`scan_project`) with `xcodebuild -list` scheme parsing
- Auto-detect build identity (`PRODUCT_BUNDLE_IDENTIFIER`, `DEVELOPMENT_TEAM`) from build settings
- Scheme confirmation tools (hide third-party schemes, lock main scheme, re-apply identity)
- Doctor panel (Xcode/Ruby/Bundler/Fastlane/CocoaPods/Gemfile checks)
- Config form and preview panel
- Skill bootstrap generator (`generate_fastlane_files`) that calls `ios-fastlane-skill/scripts/bootstrap_fastlane.sh`
  and generates:
  - `fastlane/Fastfile`
  - `fastlane/Appfile`
  - `fastlane/Pluginfile`
  - `fastlane/.env.fastlane.example`
  - `fastlane/.env.fastlane.staging.example`
  - `fastlane/.env.fastlane.prod.example`
- Bootstrap modes in UI:
  - `standard`
  - `dry-run`
  - `config file`
  - `interactive`
- Lane runner (`run_lane`) for `bundle exec fastlane ios <lane>`
- One-click `bundle install + validate_config` action
- Structured generate result in UI (per-file `exists` / `generated` status)
- Profile persistence (`save_profile` / `load_profile`) at `.fastlane-desktop/profile.json`
- Project path picker via native dialog (`Browse`)

## Tech stack

- Tauri 2
- React 18
- TypeScript
- Vite
- Rust (Tauri backend commands)

## Quick start

1. Install dependencies

```bash
npm install
```

2. Run in desktop dev mode

```bash
npm run tauri dev
```

3. Build desktop app

```bash
npm run tauri build
```

4. Stable desktop build with logs and one retry for DMG bundling

```bash
npm run build:desktop
```

Logs are written to `build-logs/tauri-build-*.log`.
If DMG packaging remains unstable after retry, the script falls back to `app` bundle build automatically.

5. Run smoke checklist (desktop build/check flow)

```bash
npm run smoke:check
```

6. Run smoke checklist with a real iOS sample project

```bash
bash scripts/smoke_check.sh --ios-project /abs/path/to/iOS/project
```

Optional: include real `dev` lane execution.

```bash
bash scripts/smoke_check.sh --ios-project /abs/path/to/iOS/project --run-dev
```

## Mac App Store Release

1. Generate macOS icon asset (`.icns`)

```bash
bash scripts/generate_macos_icons.sh /abs/path/to/1024x1024.png
```

2. Read and follow release checklist

`docs/release/MAC_APP_STORE_RELEASE_CHECKLIST.md`

Also prepare:

- `docs/release/APP_STORE_REVIEW_NOTES_TEMPLATE.md`
- `docs/release/PRIVACY_DISCLOSURE_TEMPLATE.md`

3. Run MAS release script (build/sign/pkg/upload)

```bash
bash scripts/release_mas.sh --help
```

## Directory

- `src/`: React UI
- `src-tauri/`: Rust backend commands and Tauri config

## Notes

- `run_lane` requires local Ruby/Bundler/Fastlane environment to be ready.
- `generate_fastlane_files` requires local skill path:
  `/Users/newdroid/.codex/skills/ios-fastlane-skill/scripts/bootstrap_fastlane.sh`
- The smoke script applies a Bash 3.2 compatibility patch to generated `scripts/doctor_fastlane_env.sh` in sample iOS projects.
- MAS signing/upload script requires Apple certificates and provisioning profile prepared in Keychain.
