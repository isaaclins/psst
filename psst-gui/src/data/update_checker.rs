use druid::{Data, Lens};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
            if !self.download_urls.windows.is_empty() {
                return Some(self.download_urls.windows.as_str());
            }
        }

        #[cfg(target_os = "macos")]
        {
            if !self.download_urls.macos.is_empty() {
                return Some(self.download_urls.macos.as_str());
            }
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            if !self.download_urls.linux_x86_64.is_empty() {
                return Some(self.download_urls.linux_x86_64.as_str());
            }
        }

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            if !self.download_urls.linux_aarch64.is_empty() {
                return Some(self.download_urls.linux_aarch64.as_str());
            }
        }

        None
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
}
