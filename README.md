# iOS Fastlane Desktop (Prototype)

A macOS desktop prototype built with Tauri + React + TypeScript for visual fastlane configuration and lane execution.

## Why this exists

This project turns common iOS fastlane workflows into a GUI:

- Detect project basics (`xcworkspace` / `xcodeproj`)
- Configure signing, distribution and quality flags
- Generate fastlane env files
- Run lanes and inspect logs in-app

## Current prototype scope

- Project scanner command (`scan_project`)
- Config form and preview panel
- Fastlane env generator (`generate_fastlane_files`)
- Lane runner (`run_lane`) for `bundle exec fastlane ios <lane>`

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

## Directory

- `src/`: React UI
- `src-tauri/`: Rust backend commands and Tauri config

## Notes

- This is an MVP prototype.
- `scan_project` currently returns a basic scheme placeholder and should be expanded with real `xcodebuild -list` parsing in next iteration.
- `run_lane` requires local Ruby/Bundler/Fastlane environment to be ready.

## Next steps

- Integrate existing `ios-fastlane-skill` templates directly
- Add config profile save/load
- Add doctor checks (Xcode/Ruby/Bundler/Fastlane)
- Add richer lane presets and failure diagnostics
