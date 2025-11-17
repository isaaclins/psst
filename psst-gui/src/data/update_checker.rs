use druid::{Data, Lens};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use url::Url;

const GITHUB_API_URL: &str = "https://api.github.com/repos/isaaclins/psst/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub struct UpdateInfo {
    pub version: String,
    pub release_url: String,
    pub release_notes: String,
    pub download_urls: DownloadUrls,
}

#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub struct DownloadUrls {
    pub windows: String,
    pub macos: String,
    pub linux_x86_64: String,
    pub linux_aarch64: String,
    pub deb_amd64: String,
    pub deb_arm64: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    body: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Clone, Debug, Data, PartialEq, Eq)]
pub enum UpdateInstallPhase {
    Starting,
    Downloading,
    Installing,
    Success,
    Error,
}

#[derive(Clone, Data)]
pub struct UpdateInstallEvent {
    pub phase: UpdateInstallPhase,
    pub message: String,
}

impl UpdateInstallEvent {
    pub fn new(phase: UpdateInstallPhase, message: impl Into<String>) -> Self {
        Self {
            phase,
            message: message.into(),
        }
    }
}

pub struct UpdateInstaller;

impl UpdateInfo {
    /// Check if there's a new version available by querying GitHub API
    pub fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
        log::info!("Checking for updates from GitHub...");

        let mut response = ureq::get(GITHUB_API_URL)
            .call()
            .map_err(|e| format!("Failed to fetch release info: {}", e))?;

        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let release: GitHubRelease = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse release info: {}", e))?;

        log::info!("Latest version from GitHub: {}", release.tag_name);
        log::info!("Current version: {}", CURRENT_VERSION);

        // Check if the release version is different from current
        if Self::is_newer_version(&release.tag_name, CURRENT_VERSION) {
            let download_urls = Self::extract_download_urls(&release.assets);
            Ok(Some(UpdateInfo {
                version: release.tag_name,
                release_url: release.html_url,
                release_notes: release.body,
                download_urls,
            }))
        } else {
            log::info!("No updates available");
            Ok(None)
        }
    }

    /// Compare version strings to determine if remote version is newer
    fn is_newer_version(remote: &str, current: &str) -> bool {
        // Remove 'v' prefix if present
        let remote = remote.trim_start_matches('v');
        let current = current.trim_start_matches('v');

        // For date-based versions like "2025.11.15-abc1234"
        // Just do a string comparison since they're chronologically ordered
        remote > current
    }

    /// Extract download URLs from GitHub release assets
    fn extract_download_urls(assets: &[GitHubAsset]) -> DownloadUrls {
        let mut urls = DownloadUrls {
            windows: String::new(),
            macos: String::new(),
            linux_x86_64: String::new(),
            linux_aarch64: String::new(),
            deb_amd64: String::new(),
            deb_arm64: String::new(),
        };

        for asset in assets {
            match asset.name.as_str() {
                "Psst.exe" => urls.windows = asset.browser_download_url.clone(),
                "Psst.dmg" => urls.macos = asset.browser_download_url.clone(),
                "psst-linux-x86_64" => urls.linux_x86_64 = asset.browser_download_url.clone(),
                "psst-linux-aarch64" => urls.linux_aarch64 = asset.browser_download_url.clone(),
                "psst-amd64.deb" => urls.deb_amd64 = asset.browser_download_url.clone(),
                "psst-arm64.deb" => urls.deb_arm64 = asset.browser_download_url.clone(),
                _ => {}
            }
        }

        urls
    }

    /// Get the appropriate download URL for the current platform
    pub fn get_platform_download_url(&self) -> Option<&str> {
        #[cfg(target_os = "windows")]
        {
            return self.get_download_url_for_platform(UpdatePlatform::Windows);
        }

        #[cfg(target_os = "macos")]
        {
            return self.get_download_url_for_platform(UpdatePlatform::Macos);
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return self.get_download_url_for_platform(UpdatePlatform::LinuxX86_64);
        }

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            return self.get_download_url_for_platform(UpdatePlatform::LinuxAarch64);
        }

        #[allow(unreachable_code)]
        {
            None
        }
    }

    pub fn get_download_url_for_platform(&self, platform: UpdatePlatform) -> Option<&str> {
        match platform {
            UpdatePlatform::Windows => empty_to_none(&self.download_urls.windows),
            UpdatePlatform::Macos => empty_to_none(&self.download_urls.macos),
            UpdatePlatform::LinuxX86_64 => empty_to_none(&self.download_urls.linux_x86_64),
            UpdatePlatform::LinuxAarch64 => empty_to_none(&self.download_urls.linux_aarch64),
            UpdatePlatform::DebAmd64 => empty_to_none(&self.download_urls.deb_amd64),
            UpdatePlatform::DebArm64 => empty_to_none(&self.download_urls.deb_arm64),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdatePlatform {
    Windows,
    Macos,
    LinuxX86_64,
    LinuxAarch64,
    DebAmd64,
    DebArm64,
}

fn empty_to_none(value: &str) -> Option<&str> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

impl UpdateInstaller {
    pub fn download_and_install<F>(info: &UpdateInfo, mut notify: F) -> Result<(), String>
    where
        F: FnMut(UpdateInstallPhase, &str),
    {
        notify(UpdateInstallPhase::Downloading, "Downloading update...");
        let download_path = Self::download_update_payload(info)?;

        let installing_message = format!("Installing update {}...", info.version);
        notify(UpdateInstallPhase::Installing, &installing_message);

        let install_result = Self::install_downloaded_payload(info, &download_path);
        let _ = fs::remove_file(&download_path);

        install_result
    }

    fn download_update_payload(info: &UpdateInfo) -> Result<PathBuf, String> {
        let url = info
            .get_platform_download_url()
            .ok_or_else(|| "No download available for this platform".to_string())?;

        let parsed_url = Url::parse(url).map_err(|e| format!("Invalid download URL: {}", e))?;

        let file_name = parsed_url
            .path_segments()
            .and_then(|segments| segments.last())
            .filter(|segment| !segment.is_empty())
            .unwrap_or("psst-update.bin");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let temp_file_name = format!("psst-update-{}-{}", timestamp, file_name);
        let temp_path = env::temp_dir().join(temp_file_name);

        log::info!(
            "Downloading update {} from {} to {}",
            info.version,
            url,
            temp_path.display()
        );

        let response = ureq::get(url)
            .call()
            .map_err(|e| format!("Failed to download update: {}", e))?;

        let mut reader = response.into_body().into_reader();
        let mut file = File::create(&temp_path)
            .map_err(|e| format!("Failed to create temporary file: {}", e))?;

        io::copy(&mut reader, &mut file)
            .map_err(|e| format!("Failed to write update payload: {}", e))?;
        file.flush()
            .map_err(|e| format!("Failed to flush update payload: {}", e))?;

        Ok(temp_path)
    }

    fn install_downloaded_payload(info: &UpdateInfo, path: &Path) -> Result<(), String> {
        Self::install_platform_payload(info, path)
    }

    #[cfg(target_os = "macos")]
    fn install_platform_payload(_info: &UpdateInfo, path: &Path) -> Result<(), String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mount_dir = env::temp_dir().join(format!("psst-update-mount-{}", timestamp));
        fs::create_dir_all(&mount_dir)
            .map_err(|e| format!("Failed to create mount point: {}", e))?;

        let attach_status = Command::new("hdiutil")
            .arg("attach")
            .arg(path)
            .arg("-nobrowse")
            .arg("-mountpoint")
            .arg(&mount_dir)
            .status()
            .map_err(|e| format!("Failed to mount update image: {}", e))?;

        if !attach_status.success() {
            return Err(format!(
                "Failed to mount update image (exit code {:?})",
                attach_status.code()
            ));
        }

        struct MountGuard {
            mount_point: PathBuf,
        }

        impl Drop for MountGuard {
            fn drop(&mut self) {
                if let Err(err) = Command::new("hdiutil")
                    .arg("detach")
                    .arg(&self.mount_point)
                    .arg("-quiet")
                    .status()
                {
                    log::warn!("Failed to detach update image: {}", err);
                }
                if let Err(err) = fs::remove_dir_all(&self.mount_point) {
                    log::warn!("Failed to remove temporary mount point: {}", err);
                }
            }
        }

        let mount_guard = MountGuard {
            mount_point: mount_dir.clone(),
        };

        let app_bundle = mount_dir.join("Psst.app");
        if !app_bundle.exists() {
            return Err("Mounted image does not contain Psst.app".into());
        }

        let applications_dir = Path::new("/Applications/Psst.app");
        if applications_dir.exists() {
            fs::remove_dir_all(applications_dir)
                .map_err(|e| format!("Failed to remove existing installation: {}", e))?;
        }

        let copy_status = Command::new("cp")
            .arg("-R")
            .arg(&app_bundle)
            .arg("/Applications/")
            .status()
            .map_err(|e| format!("Failed to copy new application bundle: {}", e))?;

        if !copy_status.success() {
            return Err(format!(
                "Failed to copy new application bundle (exit code {:?})",
                copy_status.code()
            ));
        }

        if let Err(err) = Command::new("xattr")
            .arg("-dr")
            .arg("com.apple.quarantine")
            .arg("/Applications/Psst.app/")
            .status()
        {
            log::warn!("Failed to remove quarantine flag: {}", err);
        }

        if let Err(err) = Command::new("xattr")
            .arg("-l")
            .arg("/Applications/Psst.app/")
            .status()
        {
            log::warn!("Failed to list xattr for Psst.app: {}", err);
        }

        drop(mount_guard);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn install_platform_payload(_info: &UpdateInfo, path: &Path) -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;

        let current_exe = env::current_exe()
            .map_err(|e| format!("Failed to determine current executable: {}", e))?;
        let target_dir = current_exe
            .parent()
            .ok_or_else(|| "Failed to determine installation directory".to_string())?;

        let file_name = current_exe
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("psst");

        let staging = target_dir.join(format!("{}.update", file_name));

        if staging.exists() {
            fs::remove_file(&staging)
                .map_err(|e| format!("Failed to remove stale staging file: {}", e))?;
        }

        fs::copy(path, &staging).map_err(|e| format!("Failed to stage updated binary: {}", e))?;

        fs::set_permissions(&staging, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("Failed to set permissions on staged binary: {}", e))?;

        fs::rename(&staging, &current_exe)
            .map_err(|e| format!("Failed to replace current binary: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn install_platform_payload(_info: &UpdateInfo, path: &Path) -> Result<(), String> {
        let current_exe = env::current_exe()
            .map_err(|e| format!("Failed to determine current executable: {}", e))?;
        let target_dir = current_exe
            .parent()
            .ok_or_else(|| "Failed to determine installation directory".to_string())?;

        let staged_path = target_dir.join("Psst.update.exe");

        if staged_path.exists() {
            fs::remove_file(&staged_path)
                .map_err(|e| format!("Failed to remove stale staged update: {}", e))?;
        }

        fs::copy(path, &staged_path)
            .map_err(|e| format!("Failed to stage updated executable: {}", e))?;

        let pid = std::process::id();
        let staged = staged_path
            .to_str()
            .ok_or_else(|| "Staged path contains invalid unicode".to_string())?
            .replace('"', "\"");
        let target = current_exe
            .to_str()
            .ok_or_else(|| "Executable path contains invalid unicode".to_string())?
            .replace('"', "\"");

        let script = format!(
            "$ErrorActionPreference='Stop'; Wait-Process -Id {pid}; Copy-Item -Path \"{staged}\" -Destination \"{target}\" -Force; Remove-Item -Path \"{staged}\" -Force",
            pid = pid,
            staged = staged,
            target = target,
        );

        Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .spawn()
            .map_err(|e| format!("Failed to schedule update replacement: {}", e))?;

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn install_platform_payload(_info: &UpdateInfo, _path: &Path) -> Result<(), String> {
        Err("Automatic installation is not supported on this platform".into())
    }
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize)]
pub struct UpdatePreferences {
    /// Whether to check for updates on startup
    pub check_on_startup: bool,
    /// Timestamp of the last update check (seconds since UNIX epoch)
    pub last_check_timestamp: u64,
    /// Version that the user has dismissed (won't show notification again for this version)
    pub dismissed_version: Option<String>,
}

impl Default for UpdatePreferences {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            last_check_timestamp: 0,
            dismissed_version: None,
        }
    }
}

impl UpdatePreferences {
    /// Check if enough time has passed since the last update check
    pub fn should_check_for_updates(&self) -> bool {
        if !self.check_on_startup {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let time_since_last_check = now.saturating_sub(self.last_check_timestamp);

        time_since_last_check >= UPDATE_CHECK_INTERVAL.as_secs()
    }

    /// Update the timestamp to now
    pub fn mark_checked(&mut self) {
        self.last_check_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }

    /// Check if a version has been dismissed
    pub fn is_version_dismissed(&self, version: &str) -> bool {
        self.dismissed_version
            .as_ref()
            .map(|v| v == version)
            .unwrap_or(false)
    }

    /// Dismiss a specific version
    pub fn dismiss_version(&mut self, version: String) {
        self.dismissed_version = Some(version);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(UpdateInfo::is_newer_version("2025.11.16", "2025.11.15"));
        assert!(UpdateInfo::is_newer_version("2025.12.01", "2025.11.30"));
        assert!(!UpdateInfo::is_newer_version("2025.11.15", "2025.11.15"));
        assert!(!UpdateInfo::is_newer_version("2025.11.14", "2025.11.15"));
    }

    #[test]
    fn test_version_with_prefix() {
        assert!(UpdateInfo::is_newer_version("v2025.11.16", "0.1.0"));
        assert!(UpdateInfo::is_newer_version("2025.11.16", "v0.1.0"));
    }

    #[test]
    fn test_should_check_for_updates() {
        let mut prefs = UpdatePreferences::default();

        // Should check on first run
        assert!(prefs.should_check_for_updates());

        // Mark as checked
        prefs.mark_checked();

        // Should not check immediately after
        assert!(!prefs.should_check_for_updates());

        // Simulate 25 hours passed
        prefs.last_check_timestamp -= 25 * 60 * 60;
        assert!(prefs.should_check_for_updates());
    }

    #[test]
    fn test_dismiss_version() {
        let mut prefs = UpdatePreferences::default();

        assert!(!prefs.is_version_dismissed("2025.11.15"));

        prefs.dismiss_version("2025.11.15".to_string());

        assert!(prefs.is_version_dismissed("2025.11.15"));
        assert!(!prefs.is_version_dismissed("2025.11.16"));
    }

    #[test]
    fn test_extract_download_urls() {
        let assets = vec![
            GitHubAsset {
                name: "Psst.exe".to_string(),
                browser_download_url: "https://example.com/Psst.exe".to_string(),
            },
            GitHubAsset {
                name: "Psst.dmg".to_string(),
                browser_download_url: "https://example.com/Psst.dmg".to_string(),
            },
        ];

        let urls = UpdateInfo::extract_download_urls(&assets);

        assert_eq!(urls.windows, "https://example.com/Psst.exe");
        assert_eq!(urls.macos, "https://example.com/Psst.dmg");
        assert!(urls.linux_x86_64.is_empty());
    }

    fn sample_update_info() -> UpdateInfo {
        UpdateInfo {
            version: "2025.11.17".into(),
            release_url: "https://example.com/release".into(),
            release_notes: String::new(),
            download_urls: DownloadUrls {
                windows: "https://example.com/Psst.exe".into(),
                macos: "https://example.com/Psst.dmg".into(),
                linux_x86_64: "https://example.com/psst-linux-x86_64".into(),
                linux_aarch64: "https://example.com/psst-linux-aarch64".into(),
                deb_amd64: "https://example.com/psst-amd64.deb".into(),
                deb_arm64: "https://example.com/psst-arm64.deb".into(),
            },
        }
    }

    #[test]
    fn test_platform_url_lookup() {
        let info = sample_update_info();

        assert_eq!(
            info.get_download_url_for_platform(UpdatePlatform::Windows),
            Some("https://example.com/Psst.exe")
        );
        assert_eq!(
            info.get_download_url_for_platform(UpdatePlatform::Macos),
            Some("https://example.com/Psst.dmg")
        );
        assert_eq!(
            info.get_download_url_for_platform(UpdatePlatform::LinuxX86_64),
            Some("https://example.com/psst-linux-x86_64")
        );
        assert_eq!(
            info.get_download_url_for_platform(UpdatePlatform::LinuxAarch64),
            Some("https://example.com/psst-linux-aarch64")
        );
        assert_eq!(
            info.get_download_url_for_platform(UpdatePlatform::DebAmd64),
            Some("https://example.com/psst-amd64.deb")
        );
        assert_eq!(
            info.get_download_url_for_platform(UpdatePlatform::DebArm64),
            Some("https://example.com/psst-arm64.deb")
        );
    }

    #[test]
    fn test_install_requires_platform_url() {
        let mut info = sample_update_info();
        info.download_urls.macos.clear();
        info.download_urls.windows.clear();
        info.download_urls.linux_x86_64.clear();
        info.download_urls.linux_aarch64.clear();
        info.download_urls.deb_amd64.clear();
        info.download_urls.deb_arm64.clear();

        let mut notifications = Vec::new();
        let result = UpdateInstaller::download_and_install(&info, |phase, message| {
            notifications.push((phase, message.to_string()))
        });

        assert!(result.is_err());
        assert_eq!(
            notifications,
            vec![(
                UpdateInstallPhase::Downloading,
                "Downloading update...".to_string()
            )]
        );
    }
}
