# Automatic Updates

Psst includes an automatic update system that helps you stay up-to-date with the latest features and bug fixes.

## Features

### Automatic Update Checking

By default, Psst will check for updates every time you start the application. This happens in the background and won't interfere with your music listening experience.

### Manual Update Check

You can manually check for updates at any time:

1. Open **Preferences** (from the menu or keyboard shortcut)
2. Navigate to the **Updates** tab
3. Click **Check for Updates**

### Version Information

The Updates tab shows:

- Your current version
- Whether an update is available
- Release notes for new versions
- Download options for your platform

## Configuration

### Disabling Automatic Checks

If you prefer to check for updates manually:

1. Open **Preferences > Updates**
2. Uncheck **"Check for updates on startup"**

Psst will save this preference and won't check for updates automatically until you re-enable it.

### Update Frequency

When automatic checking is enabled, Psst checks for updates:

- On application startup
- No more than once per 24 hours

This ensures you're notified of new versions without excessive network requests.

## Installing Updates

When an update is available, click **Install Update** in the Updates tab and Psst will do the rest:

1. The updater downloads the latest release asset for your platform.
2. Psst installs the update in the background without blocking the UI.
3. A status message confirms success or reports any errors.

### Platform Notes

- **macOS**: The updater mounts the DMG, copies `Psst.app` into `/Applications`, removes any quarantine flags with `xattr -dr com.apple.quarantine /Applications/Psst.app/`, and verifies attributes via `xattr -l /Applications/Psst.app/`.
- **Windows**: A silent PowerShell helper stages the new `Psst.exe` and replaces the current executable after you exit the app. Restart Psst to finish the update.
- **Linux**: The staged binary replaces the currently running executable in-place. Restart Psst to run the new version.

If automatic installation is unavailable (for example on unsupported platforms), the updater offers an easy shortcut to open the release page so you can download manually.

## Dismissing Updates

If you don't want to install a specific update:

1. Click the **Dismiss** button
2. Psst won't notify you about this version again
3. You'll be notified when a newer version is released

## Update Process

1. **Check**: Psst queries the GitHub API for the latest release
2. **Compare**: The current version is compared with the latest available version
3. **Notify**: If a newer version is available (and not dismissed), you'll see it in the Updates tab
4. **Download**: You choose when to download and install the update

## Privacy & Security

- **No tracking**: Psst only checks the official GitHub repository for releases
- **HTTPS only**: All update checks use secure HTTPS connections
- **User-initiated installation**: Updates are downloaded and installed only after you click **Install Update**—no silent background updates
- **No personal data**: No personal information is sent during update checks

## Troubleshooting

### Update Check Failed

If update checking fails, possible causes include:

- No internet connection
- GitHub API temporarily unavailable
- Firewall or proxy blocking GitHub access

The error will be logged, and you can try checking again later.

### No Updates Shown

If no updates appear:

- You're running the latest version
- You've dismissed the latest version (check your config)
- The update check hasn't run yet (wait for startup or manually check)

### Update Version Format

Psst uses date-based versions in the format `YYYY.MM.DD-COMMIT`. This makes it easy to see how recent your version is.

## Manual Validation Checklist

To smoke-test the installer on each platform:

- **macOS**
  - Download the latest universal DMG from the Releases page.
  - Launch Psst and use **Preferences → Updates → Install Update** while the DMG is on disk.
  - When prompted in the logs, confirm `/Applications/Psst.app` was replaced and run `xattr -l /Applications/Psst.app/` to verify no quarantine flag remains.
- **Windows**
  - Place the newest `Psst.exe` in a writable folder alongside the running build.
  - Trigger **Install Update** and exit the app; ensure `Psst.update.exe` is created and deleted after restart, and the timestamp on `Psst.exe` is updated.
- **Linux (x86_64 & aarch64)**
  - Copy the corresponding binary next to the existing executable and run the installer.
  - Confirm the executable is replaced in-place and marked executable (`chmod +x`).

The automated unit tests cover error-handling, URL selection, and notification sequencing, but the above manual checks verify platform tooling (hdiutil, PowerShell, filesystem permissions) that cannot be exercised in CI.

## Technical Details

### GitHub Integration

Updates are fetched from the official Psst GitHub repository:

- Repository: `isaaclins/psst`
- API endpoint: `https://api.github.com/repos/isaaclins/psst/releases/latest`

### Configuration Storage

Update preferences are stored in your Psst configuration file:

- `check_on_startup`: Boolean flag for automatic checking
- `last_check_timestamp`: Unix timestamp of last check
- `dismissed_version`: Version you've chosen to ignore

### Release Assets

Each release includes pre-built binaries for:

- Windows (x86_64)
- macOS (Universal binary: x86_64 + ARM64)
- Linux x86_64 (binary and .deb)
- Linux ARM64/aarch64 (binary and .deb)

## For Developers

If you're building Psst from source, the update system will compare your version (`0.1.0`) against published releases. Since date-based versions are chronologically ordered, you'll always be notified of newer official releases.

To disable update checks during development, uncheck "Check for updates on startup" in Preferences.

## Feedback

If you encounter any issues with the update system, please report them on our [GitHub Issues](https://github.com/isaaclins/psst/issues) page.
