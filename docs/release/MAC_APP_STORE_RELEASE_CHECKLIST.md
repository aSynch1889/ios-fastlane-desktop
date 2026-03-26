# Mac App Store Release Checklist

## 1. Accounts and Certificates

- Apple Developer Program (organization or individual) is active.
- App ID matches `com.asynch.fastlane.desktop`.
- Certificates are created and installed in Keychain:
- `3rd Party Mac Developer Application: ...`
- `3rd Party Mac Developer Installer: ...`
- A Mac App Store provisioning profile is created for this App ID.

## 2. App Store Connect Setup

- App record exists in App Store Connect.
- Bundle ID exactly matches local app identifier.
- Version (`CFBundleShortVersionString`) and build (`CFBundleVersion`) are unique for the upload.
- Privacy policy URL is prepared.
- App Privacy answers are completed.

## 3. Local Packaging Prerequisites

- Xcode command line tools installed.
- `xcrun iTMSTransporter` available.
- Entitlements files are present:
- `src-tauri/entitlements/mas.plist`
- `src-tauri/entitlements/mas.inherit.plist`
- macOS icon exists:
- `src-tauri/icons/icon.icns` (generate with `bash scripts/generate_macos_icons.sh /abs/path/to/1024x1024.png`)

## 4. Build and Sign

```bash
bash scripts/release_mas.sh \
  --team-id YOUR_TEAM_ID \
  --app-cert "3rd Party Mac Developer Application: YOUR_NAME (TEAMID)" \
  --installer-cert "3rd Party Mac Developer Installer: YOUR_NAME (TEAMID)" \
  --provision-profile /abs/path/YourMacAppStore.provisionprofile \
  --skip-upload
```

Output package:

- `dist/mas/iOS Fastlane Desktop-mas.pkg`

## 5. Upload to App Store Connect

```bash
bash scripts/release_mas.sh \
  --team-id YOUR_TEAM_ID \
  --app-cert "3rd Party Mac Developer Application: YOUR_NAME (TEAMID)" \
  --installer-cert "3rd Party Mac Developer Installer: YOUR_NAME (TEAMID)" \
  --provision-profile /abs/path/YourMacAppStore.provisionprofile \
  --apple-id your@appleid.com \
  --app-password xxxx-xxxx-xxxx-xxxx \
  --provider-short-name YOUR_PROVIDER
```

`--provider-short-name` is optional; use it when your Apple ID has multiple provider contexts.

## 6. Pre-Submission QA

- Launch app from signed package on a clean macOS user.
- Verify project-folder selection and fastlane command execution still works in sandboxed context.
- Verify there is no crash on first launch.
- Confirm no development-only logs or placeholder assets are shipped.

## 7. Submission Metadata

- App description, keywords, support URL.
- Privacy policy URL.
- Screenshots (required display sizes).
- Review notes explaining:
- The app runs local developer tools (`xcodebuild`, `bundle`, `fastlane`) on user-selected project folders.
- No hidden background services.
