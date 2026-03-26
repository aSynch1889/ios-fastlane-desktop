# App Store Review Notes (Template)

## App Purpose

iOS Fastlane Desktop helps iOS developers configure and execute local fastlane workflows through a desktop UI.

## Core Behavior

- User selects a local iOS project directory.
- App inspects project metadata (`.xcodeproj` / `.xcworkspace` / schemes).
- App generates fastlane configuration files locally.
- App executes developer tooling commands (`xcodebuild`, `bundle`, `fastlane`) only on user-selected project paths.

## Data and Network

- The app does not require account login.
- The app does not upload user data by itself.
- Network access occurs only when developer-triggered lanes/plugins call external services (for example distribution endpoints configured by the user).

## Sandbox and File Access

- File access is limited to paths explicitly selected by the user.
- No hidden background daemon/service is installed.

## Test Account

Not required.
