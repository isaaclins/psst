use std::{
    env::{self, VarError},
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

use druid::{Data, Lens, Size};
use platform_dirs::AppDirs;
use psst_core::{
    audio::equalizer::{EqualizerConfig, EqualizerPreset},
    cache::{mkdir_if_not_exists, CacheHandle},
    connection::Credentials,
    player::PlaybackConfig,
    session::{SessionConfig, SessionConnection},
};
use serde::{Deserialize, Serialize};

use super::{Nav, Promise, QueueBehavior, SliderScrollScale};
use crate::ui::theme;

#[derive(Clone, Debug, Data, Lens)]
pub struct Preferences {
    pub active: PreferencesTab,
    #[data(ignore)]
    pub cache: Option<CacheHandle>,
    pub cache_size: Promise<u64, (), ()>,
    pub auth: Authentication,
    pub lastfm_auth_result: Option<String>,
}

impl Preferences {
    pub fn reset(&mut self) {
        self.cache_size.clear();
        self.auth.result.clear();
        self.auth.lastfm_api_key_input.clear();
        self.auth.lastfm_api_secret_input.clear();
    }

    pub fn measure_cache_usage() -> Option<u64> {
        Config::cache_dir().and_then(|path| get_dir_size(&path))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Data)]
pub enum PreferencesTab {
    General,
    Appearance,
    Equalizer,
    Account,
    Cache,
    About,
}

#[derive(Clone, Debug, Data, Lens)]
pub struct Authentication {
    pub username: String,
    pub password: String,
    pub access_token: String,
    pub result: Promise<(), (), String>,
    #[data(ignore)]
    pub lastfm_api_key_input: String,
    #[data(ignore)]
    pub lastfm_api_secret_input: String,
}

impl Authentication {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            access_token: String::new(),
            result: Promise::Empty,
            lastfm_api_key_input: String::new(),
            lastfm_api_secret_input: String::new(),
        }
    }

    pub fn session_config(&self) -> SessionConfig {
        SessionConfig {
            login_creds: if !self.access_token.is_empty() {
                Credentials::from_access_token(self.access_token.clone())
            } else {
                Credentials::from_username_and_password(
                    self.username.clone(),
                    self.password.clone(),
                )
            },
            proxy_url: Config::proxy(),
        }
    }

    pub fn authenticate_and_get_credentials(config: SessionConfig) -> Result<Credentials, String> {
        let connection = SessionConnection::open(config).map_err(|err| err.to_string())?;
        Ok(connection.credentials)
    }

    pub fn clear(&mut self) {
        self.username.clear();
        self.password.clear();
    }
}

const APP_NAME: &str = "Psst";
const CONFIG_FILENAME: &str = "config.json";
const PROXY_ENV_VAR: &str = "SOCKS_PROXY";

fn default_sidebar_visible() -> bool {
    true
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[data(ignore)]
    credentials: Option<Credentials>,
    #[serde(alias = "oauth_token_override")]
    pub oauth_bearer: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub audio_quality: AudioQuality,
    pub theme: Theme,
    #[serde(default)]
    pub custom_theme: CustomTheme,
    pub volume: f64,
    pub last_route: Option<Nav>,
    pub queue_behavior: QueueBehavior,
    pub show_track_cover: bool,
    pub show_playlist_images: bool,
    pub window_size: Size,
    pub slider_scroll_scale: SliderScrollScale,
    pub sort_order: SortOrder,
    pub sort_criteria: SortCriteria,
    pub paginated_limit: usize,
    pub seek_duration: usize,
    pub lastfm_session_key: Option<String>,
    pub lastfm_api_key: Option<String>,
    pub lastfm_api_secret: Option<String>,
    pub lastfm_enable: bool,
    #[serde(default = "default_sidebar_visible")]
    pub sidebar_visible: bool,
    #[data(ignore)]
    #[serde(default)]
    pub equalizer: EqualizerConfig,
    #[data(ignore)]
    #[serde(default)]
    pub custom_equalizer_presets: Vec<EqualizerPreset>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            credentials: Default::default(),
            oauth_bearer: None,
            oauth_refresh_token: None,
            audio_quality: Default::default(),
            theme: Default::default(),
            custom_theme: Default::default(),
            volume: 1.0,
            last_route: Default::default(),
            queue_behavior: Default::default(),
            show_track_cover: Default::default(),
            show_playlist_images: true,
            window_size: Size::new(theme::grid(80.0), theme::grid(100.0)),
            slider_scroll_scale: Default::default(),
            sort_order: Default::default(),
            sort_criteria: Default::default(),
            paginated_limit: 500,
            seek_duration: 10,
            lastfm_session_key: None,
            lastfm_api_key: None,
            lastfm_api_secret: None,
            lastfm_enable: false,
            sidebar_visible: true,
            equalizer: Default::default(),
            custom_equalizer_presets: Vec::new(),
        }
    }
}

impl Config {
    fn app_dirs() -> Option<AppDirs> {
        const USE_XDG_ON_MACOS: bool = false;

        AppDirs::new(Some(APP_NAME), USE_XDG_ON_MACOS)
    }

    pub fn spotify_local_files_file(username: &str) -> Option<PathBuf> {
        AppDirs::new(Some("spotify"), false).map(|dir| {
            let path = format!("Users/{username}-user/local-files.bnk");
            dir.config_dir.join(path)
        })
    }

    pub fn cache_dir() -> Option<PathBuf> {
        Self::app_dirs().map(|dirs| dirs.cache_dir)
    }

    pub fn config_dir() -> Option<PathBuf> {
        Self::app_dirs().map(|dirs| dirs.config_dir)
    }

    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join(CONFIG_FILENAME))
    }

    pub fn load() -> Option<Config> {
        let path = Self::config_path().expect("Failed to get config path");
        if let Ok(file) = File::open(&path) {
            log::info!("loading config: {:?}", &path);
            let reader = BufReader::new(file);
            Some(serde_json::from_reader(reader).expect("Failed to read config"))
        } else {
            None
        }
    }

    pub fn save(&self) {
        let dir = Self::config_dir().expect("Failed to get config dir");
        let path = Self::config_path().expect("Failed to get config path");
        mkdir_if_not_exists(&dir).expect("Failed to create config dir");

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(target_family = "unix")]
        options.mode(0o600);

        let file = options.open(&path).expect("Failed to create config");
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self).expect("Failed to write config");
        log::info!("saved config: {:?}", &path);
    }

    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    pub fn store_credentials(&mut self, credentials: Credentials) {
        self.credentials = Some(credentials);
    }

    pub fn clear_credentials(&mut self) {
        self.credentials = Default::default();
    }

    pub fn username(&self) -> Option<&str> {
        self.credentials
            .as_ref()
            .and_then(|c| c.username.as_deref())
    }

    pub fn session(&self) -> SessionConfig {
        SessionConfig {
            login_creds: self.credentials.clone().expect("Missing credentials"),
            proxy_url: Config::proxy(),
        }
    }

    pub fn playback(&self) -> PlaybackConfig {
        PlaybackConfig {
            bitrate: self.audio_quality.as_bitrate(),
            equalizer: self.equalizer.clone(),
            ..PlaybackConfig::default()
        }
    }

    pub fn proxy() -> Option<String> {
        env::var(PROXY_ENV_VAR).map_or_else(
            |err| match err {
                VarError::NotPresent => None,
                VarError::NotUnicode(_) => {
                    log::error!("proxy URL is not a valid unicode");
                    None
                }
            },
            Some,
        )
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data, Serialize, Deserialize, Default)]
pub enum AudioQuality {
    Low,
    Normal,
    #[default]
    High,
}

impl AudioQuality {
    fn as_bitrate(self) -> usize {
        match self {
            AudioQuality::Low => 96,
            AudioQuality::Normal => 160,
            AudioQuality::High => 320,
        }
    }
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, Eq, PartialEq)]
pub struct CustomTheme {
    pub background: String,
    pub surface: String,
    pub primary_text: String,
    pub accent: String,
    pub highlight: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: String,
}

fn default_font_family() -> String {
    "System UI".into()
}

fn default_font_size() -> String {
    "13.0".into()
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            background: "#1c1c1f".into(),
            surface: "#242429".into(),
            primary_text: "#f2f2f2".into(),
            accent: "#1db954".into(),
            highlight: "#3a7bd5".into(),
            font_family: default_font_family(),
            font_size: default_font_size(),
        }
    }
}

impl CustomTheme {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let theme: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        theme.validate()?;
        Ok(theme)
    }

    pub fn export_to_file(&self, path: &Path) -> Result<(), String> {
        let json = self.to_json()?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn import_from_file(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_json(&json)
    }

    fn validate(&self) -> Result<(), String> {
        // Validate colors are valid hex codes
        Self::validate_hex_color(&self.background, "background")?;
        Self::validate_hex_color(&self.surface, "surface")?;
        Self::validate_hex_color(&self.primary_text, "primary_text")?;
        Self::validate_hex_color(&self.accent, "accent")?;
        Self::validate_hex_color(&self.highlight, "highlight")?;

        // Validate font size is a valid number
        if self.font_size.parse::<f64>().is_err() {
            return Err(format!("Invalid font size: {}", self.font_size));
        }

        // Validate font size is reasonable (between 8 and 32)
        let size = self.font_size.parse::<f64>().unwrap();
        if !(8.0..=32.0).contains(&size) {
            return Err(format!("Font size must be between 8 and 32, got {}", size));
        }

        Ok(())
    }

    fn validate_hex_color(color: &str, field_name: &str) -> Result<(), String> {
        let trimmed = color.trim();
        let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);

        if hex.len() != 6 {
            return Err(format!(
                "Invalid color for {}: '{}' (expected #RRGGBB format)",
                field_name, color
            ));
        }

        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "Invalid hex color for {}: '{}' (must contain only 0-9, A-F)",
                field_name, color
            ));
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data, Serialize, Deserialize, Default)]
pub enum Theme {
    Light,
    Dark,
    #[default]
    Custom,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data, Serialize, Deserialize, Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data, Serialize, Deserialize, Default)]
pub enum SortCriteria {
    Title,
    Artist,
    Album,
    Duration,
    #[default]
    DateAdded,
}

fn get_dir_size(path: &Path) -> Option<u64> {
    fs::read_dir(path).ok()?.try_fold(0, |acc, entry| {
        let entry = entry.ok()?;
        let size = if entry.file_type().ok()?.is_dir() {
            get_dir_size(&entry.path())?
        } else {
            entry.metadata().ok()?.len()
        };
        Some(acc + size)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_theme_serialization() {
        let theme = CustomTheme::default();
        let json = theme.to_json().unwrap();
        let parsed = CustomTheme::from_json(&json).unwrap();
        assert_eq!(theme, parsed);
    }

    #[test]
    fn test_custom_theme_validation_valid() {
        let theme = CustomTheme {
            background: "#1c1c1f".into(),
            surface: "#242429".into(),
            primary_text: "#f2f2f2".into(),
            accent: "#1db954".into(),
            highlight: "#3a7bd5".into(),
            font_family: "System UI".into(),
            font_size: "13.0".into(),
        };
        assert!(theme.validate().is_ok());
    }

    #[test]
    fn test_custom_theme_validation_invalid_color() {
        let json = r##"{
            "background": "invalid",
            "surface": "#242429",
            "primary_text": "#f2f2f2",
            "accent": "#1db954",
            "highlight": "#3a7bd5",
            "font_family": "System UI",
            "font_size": "13.0"
        }"##;
        assert!(CustomTheme::from_json(json).is_err());
    }

    #[test]
    fn test_custom_theme_validation_invalid_font_size() {
        let json = r##"{
            "background": "#1c1c1f",
            "surface": "#242429",
            "primary_text": "#f2f2f2",
            "accent": "#1db954",
            "highlight": "#3a7bd5",
            "font_family": "System UI",
            "font_size": "invalid"
        }"##;
        assert!(CustomTheme::from_json(json).is_err());
    }

    #[test]
    fn test_custom_theme_validation_font_size_out_of_range() {
        let json = r##"{
            "background": "#1c1c1f",
            "surface": "#242429",
            "primary_text": "#f2f2f2",
            "accent": "#1db954",
            "highlight": "#3a7bd5",
            "font_family": "System UI",
            "font_size": "50.0"
        }"##;
        assert!(CustomTheme::from_json(json).is_err());
    }

    #[test]
    fn test_custom_theme_backwards_compatibility() {
        // Test that old themes without font fields can still be loaded
        let json = r##"{
            "background": "#1c1c1f",
            "surface": "#242429",
            "primary_text": "#f2f2f2",
            "accent": "#1db954",
            "highlight": "#3a7bd5"
        }"##;
        let theme = CustomTheme::from_json(json).unwrap();
        assert_eq!(theme.font_family, "System UI");
        assert_eq!(theme.font_size, "13.0");
    }
}
