use std::{
    env::{self, VarError},
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

#[cfg(target_family = "unix")]
use std::os::unix::fs::OpenOptionsExt;

use druid::{keyboard_types::Code, Data, HotKey, KbKey, Lens, RawMods, Size};
use platform_dirs::AppDirs;
use psst_core::{
    audio::equalizer::{EqualizerConfig, EqualizerPreset},
    cache::{mkdir_if_not_exists, CacheHandle},
    connection::Credentials,
    player::PlaybackConfig,
    session::{SessionConfig, SessionConnection},
};
use serde::{Deserialize, Serialize};

use super::{Nav, Promise, QueueBehavior, SliderScrollScale, UpdateInfo, UpdatePreferences};
use crate::ui::theme;

#[derive(Clone, Debug, Data, Lens)]
pub struct Preferences {
    pub active: PreferencesTab,
    #[data(ignore)]
    pub cache: Option<CacheHandle>,
    pub cache_size: Promise<u64, (), ()>,
    pub auth: Authentication,
    pub lastfm_auth_result: Option<String>,
    pub available_update: Option<UpdateInfo>,
    pub checking_update: bool,
    pub installing_update: bool,
    pub update_install_status: Option<String>,
    pub active_keybind_capture: Option<KeybindAction>,
    pub keybind_capture_display: Option<String>,
    pub keybind_capture_error: Option<String>,
    pub keybind_menu_revision: u64,
}

impl Preferences {
    pub fn reset(&mut self) {
        self.cache_size.clear();
        self.auth.result.clear();
        self.auth.lastfm_api_key_input.clear();
        self.auth.lastfm_api_secret_input.clear();
        self.active_keybind_capture = None;
        self.keybind_capture_display = None;
        self.keybind_capture_error = None;
        self.keybind_menu_revision = self.keybind_menu_revision.wrapping_add(1);
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
    Keybinds,
    Account,
    DiscordPresence,
    Cache,
    Updates,
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

/// Represents a keyboard modifier key
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

/// Represents a single keybind action
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Data)]
pub enum KeybindAction {
    // Playback controls
    PlayPause,
    Play,
    Pause,
    Next,
    Previous,
    SeekForward,
    SeekBackward,
    VolumeUp,
    VolumeDown,
    Stop,

    // Navigation
    NavigateHome,
    NavigateSavedTracks,
    NavigateSavedAlbums,
    NavigateShows,
    NavigateSearch,
    NavigateBack,
    NavigateRefresh,

    // UI Controls
    ToggleSidebar,
    ToggleLyrics,
    OpenPreferences,
    CloseWindow,
    ToggleFinder,
    FocusSearch,

    // Queue controls
    QueueBehaviorSequential,
    QueueBehaviorRandom,
    QueueBehaviorLoopTrack,
    QueueBehaviorLoopAll,
}

impl KeybindAction {
    pub fn display_name(&self) -> &'static str {
        match self {
            KeybindAction::PlayPause => "Play/Pause",
            KeybindAction::Play => "Play",
            KeybindAction::Pause => "Pause",
            KeybindAction::Next => "Next Track",
            KeybindAction::Previous => "Previous Track",
            KeybindAction::SeekForward => "Seek Forward",
            KeybindAction::SeekBackward => "Seek Backward",
            KeybindAction::VolumeUp => "Volume Up",
            KeybindAction::VolumeDown => "Volume Down",
            KeybindAction::Stop => "Stop Playback",
            KeybindAction::NavigateHome => "Navigate to Home",
            KeybindAction::NavigateSavedTracks => "Navigate to Saved Tracks",
            KeybindAction::NavigateSavedAlbums => "Navigate to Saved Albums",
            KeybindAction::NavigateShows => "Navigate to Shows",
            KeybindAction::NavigateSearch => "Navigate to Search",
            KeybindAction::NavigateBack => "Navigate Back",
            KeybindAction::NavigateRefresh => "Refresh Current Page",
            KeybindAction::ToggleSidebar => "Toggle Sidebar",
            KeybindAction::ToggleLyrics => "Toggle Lyrics",
            KeybindAction::OpenPreferences => "Open Preferences",
            KeybindAction::CloseWindow => "Close Window",
            KeybindAction::ToggleFinder => "Toggle Find in Page",
            KeybindAction::FocusSearch => "Focus Search Box",
            KeybindAction::QueueBehaviorSequential => "Queue: Sequential",
            KeybindAction::QueueBehaviorRandom => "Queue: Random",
            KeybindAction::QueueBehaviorLoopTrack => "Queue: Loop Track",
            KeybindAction::QueueBehaviorLoopAll => "Queue: Loop All",
        }
    }
}

/// Represents a key combination (key + modifiers)
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyCombination {
    pub key: String,
    pub code: Option<String>,
    pub modifiers: Vec<KeyModifier>,
}

impl KeyCombination {
    pub fn new(key: String, code: Option<String>, modifiers: Vec<KeyModifier>) -> Self {
        Self {
            key,
            code,
            modifiers,
        }
    }

    pub fn from_key_event(key_event: &druid::KeyEvent) -> Option<Self> {
        if Self::is_modifier_key(&key_event.key) {
            return None;
        }

        let key = match &key_event.key {
            KbKey::Character(ref c) => {
                if c == " " {
                    "Space".to_string()
                } else {
                    c.clone()
                }
            }
            other => format!("{:?}", other),
        };

        let code = match key_event.code {
            Code::Unidentified => None,
            other => Some(format!("{:?}", other)),
        };

        let mut modifiers = Vec::new();
        if key_event.mods.ctrl() {
            modifiers.push(KeyModifier::Ctrl);
        }
        if key_event.mods.alt() {
            modifiers.push(KeyModifier::Alt);
        }
        if key_event.mods.shift() {
            modifiers.push(KeyModifier::Shift);
        }
        if key_event.mods.meta() {
            modifiers.push(KeyModifier::Meta);
        }

        Some(Self::new(key, code, modifiers))
    }

    fn is_modifier_key(key: &KbKey) -> bool {
        matches!(
            key,
            KbKey::Shift
                | KbKey::Alt
                | KbKey::Control
                | KbKey::Meta
                | KbKey::Super
                | KbKey::Hyper
                | KbKey::CapsLock
        )
    }

    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();

        for modifier in &self.modifiers {
            match modifier {
                KeyModifier::Ctrl => parts.push("Ctrl"),
                KeyModifier::Alt => parts.push("Alt"),
                KeyModifier::Shift => parts.push("Shift"),
                KeyModifier::Meta => {
                    #[cfg(target_os = "macos")]
                    parts.push("Cmd");
                    #[cfg(not(target_os = "macos"))]
                    parts.push("Win");
                }
            }
        }

        parts.push(&self.key);
        parts.join("+")
    }

    pub fn to_hot_key(&self) -> Option<HotKey> {
        let key = self.to_kb_key();
        let mods = self.to_raw_mods();
        Some(HotKey::new(mods, key))
    }

    fn to_kb_key(&self) -> KbKey {
        match self.key.as_str() {
            "Space" => KbKey::Character(" ".into()),
            "ArrowLeft" => KbKey::ArrowLeft,
            "ArrowRight" => KbKey::ArrowRight,
            "ArrowUp" => KbKey::ArrowUp,
            "ArrowDown" => KbKey::ArrowDown,
            "Backspace" => KbKey::Backspace,
            "Delete" => KbKey::Delete,
            "Enter" | "Return" => KbKey::Enter,
            "Tab" => KbKey::Tab,
            "Escape" => KbKey::Escape,
            other => KbKey::Character(other.to_string()),
        }
    }

    fn to_raw_mods(&self) -> Option<RawMods> {
        let has_ctrl = self.modifiers.contains(&KeyModifier::Ctrl);
        let has_alt = self.modifiers.contains(&KeyModifier::Alt);
        let has_shift = self.modifiers.contains(&KeyModifier::Shift);
        let has_meta = self.modifiers.contains(&KeyModifier::Meta);

        match (has_alt, has_ctrl, has_meta, has_shift) {
            (false, false, false, false) => None,
            (true, false, false, false) => Some(RawMods::Alt),
            (false, true, false, false) => Some(RawMods::Ctrl),
            (false, false, true, false) => Some(RawMods::Meta),
            (false, false, false, true) => Some(RawMods::Shift),
            (true, true, false, false) => Some(RawMods::AltCtrl),
            (true, false, true, false) => Some(RawMods::AltMeta),
            (true, false, false, true) => Some(RawMods::AltShift),
            (false, true, true, false) => Some(RawMods::CtrlMeta),
            (false, true, false, true) => Some(RawMods::CtrlShift),
            (false, false, true, true) => Some(RawMods::MetaShift),
            (true, true, true, false) => Some(RawMods::AltCtrlMeta),
            (true, true, false, true) => Some(RawMods::AltCtrlShift),
            (true, false, true, true) => Some(RawMods::AltMetaShift),
            (false, true, true, true) => Some(RawMods::CtrlMetaShift),
            (true, true, true, true) => Some(RawMods::AltCtrlMetaShift),
        }
    }

    pub fn matches(&self, key_event: &druid::KeyEvent) -> bool {
        // Check if the key matches
        let key_matches = match &key_event.key {
            KbKey::Character(c) => c.as_str() == self.key,
            _ => self.key == format!("{:?}", key_event.key),
        };

        // Check if the code matches (if specified)
        let code_matches = if let Some(ref expected_code) = self.code {
            format!("{:?}", key_event.code) == *expected_code
        } else {
            true
        };

        if !key_matches && !code_matches {
            return false;
        }

        // Check modifiers
        let has_ctrl = self.modifiers.contains(&KeyModifier::Ctrl);
        let has_alt = self.modifiers.contains(&KeyModifier::Alt);
        let has_shift = self.modifiers.contains(&KeyModifier::Shift);
        let has_meta = self.modifiers.contains(&KeyModifier::Meta);

        key_event.mods.ctrl() == has_ctrl
            && key_event.mods.alt() == has_alt
            && key_event.mods.shift() == has_shift
            && key_event.mods.meta() == has_meta
    }
}

/// Represents a single keybind mapping
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Keybind {
    pub action: KeybindAction,
    pub combination: KeyCombination,
}

/// Configuration for all keybinds
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeybindsConfig {
    pub keybinds: Vec<Keybind>,
}

impl KeybindsConfig {
    pub fn default_keybinds() -> Self {
        Self {
            keybinds: vec![
                // Playback controls
                Keybind {
                    action: KeybindAction::PlayPause,
                    combination: KeyCombination::new("Space".into(), Some("Space".into()), vec![]),
                },
                Keybind {
                    action: KeybindAction::Next,
                    combination: KeyCombination::new(
                        "ArrowRight".into(),
                        Some("ArrowRight".into()),
                        vec![KeyModifier::Shift],
                    ),
                },
                Keybind {
                    action: KeybindAction::Previous,
                    combination: KeyCombination::new(
                        "ArrowLeft".into(),
                        Some("ArrowLeft".into()),
                        vec![KeyModifier::Shift],
                    ),
                },
                Keybind {
                    action: KeybindAction::SeekForward,
                    combination: KeyCombination::new(
                        "ArrowRight".into(),
                        Some("ArrowRight".into()),
                        vec![],
                    ),
                },
                Keybind {
                    action: KeybindAction::SeekBackward,
                    combination: KeyCombination::new(
                        "ArrowLeft".into(),
                        Some("ArrowLeft".into()),
                        vec![],
                    ),
                },
                Keybind {
                    action: KeybindAction::VolumeUp,
                    combination: KeyCombination::new("+".into(), None, vec![]),
                },
                Keybind {
                    action: KeybindAction::VolumeDown,
                    combination: KeyCombination::new("-".into(), None, vec![]),
                },
                // Navigation
                Keybind {
                    action: KeybindAction::NavigateHome,
                    combination: KeyCombination::new(
                        "h".into(),
                        Some("KeyH".into()),
                        vec![KeyModifier::Ctrl],
                    ),
                },
                Keybind {
                    action: KeybindAction::NavigateSavedTracks,
                    combination: KeyCombination::new(
                        "t".into(),
                        Some("KeyT".into()),
                        vec![KeyModifier::Ctrl],
                    ),
                },
                Keybind {
                    action: KeybindAction::NavigateSavedAlbums,
                    combination: KeyCombination::new(
                        "a".into(),
                        Some("KeyA".into()),
                        vec![KeyModifier::Ctrl],
                    ),
                },
                Keybind {
                    action: KeybindAction::NavigateShows,
                    combination: KeyCombination::new(
                        "p".into(),
                        Some("KeyP".into()),
                        vec![KeyModifier::Ctrl],
                    ),
                },
                Keybind {
                    action: KeybindAction::NavigateSearch,
                    combination: KeyCombination::new(
                        "f".into(),
                        Some("KeyF".into()),
                        vec![KeyModifier::Ctrl],
                    ),
                },
                Keybind {
                    action: KeybindAction::NavigateBack,
                    combination: KeyCombination::new(
                        "Backspace".into(),
                        Some("Backspace".into()),
                        vec![],
                    ),
                },
                Keybind {
                    action: KeybindAction::NavigateRefresh,
                    combination: KeyCombination::new(
                        "r".into(),
                        Some("KeyR".into()),
                        vec![KeyModifier::Meta],
                    ),
                },
                // UI Controls
                Keybind {
                    action: KeybindAction::ToggleLyrics,
                    combination: KeyCombination::new(
                        "l".into(),
                        Some("KeyL".into()),
                        vec![KeyModifier::Ctrl],
                    ),
                },
                Keybind {
                    action: KeybindAction::OpenPreferences,
                    combination: KeyCombination::new(",".into(), None, vec![KeyModifier::Meta]),
                },
                Keybind {
                    action: KeybindAction::ToggleFinder,
                    combination: KeyCombination::new(
                        "f".into(),
                        Some("KeyF".into()),
                        vec![KeyModifier::Meta],
                    ),
                },
            ],
        }
    }

    pub fn find_action(&self, key_event: &druid::KeyEvent) -> Option<KeybindAction> {
        for keybind in &self.keybinds {
            if keybind.combination.matches(key_event) {
                return Some(keybind.action.clone());
            }
        }
        None
    }

    pub fn conflicting_action(
        &self,
        combination: &KeyCombination,
        exclude_action: KeybindAction,
    ) -> Option<KeybindAction> {
        self.keybinds
            .iter()
            .find(|keybind| keybind.action != exclude_action && keybind.combination == *combination)
            .map(|keybind| keybind.action)
    }

    pub fn set_keybind(&mut self, action: KeybindAction, combination: KeyCombination) {
        if let Some(existing) = self
            .keybinds
            .iter_mut()
            .find(|keybind| keybind.action == action)
        {
            existing.combination = combination;
        } else {
            self.keybinds.push(Keybind {
                action,
                combination,
            });
        }
    }

    pub fn get_keybind(&self, action: KeybindAction) -> Option<&KeyCombination> {
        self.keybinds
            .iter()
            .find(|keybind| keybind.action == action)
            .map(|keybind| &keybind.combination)
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default_keybinds();
    }
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self::default_keybinds()
    }
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
    #[serde(default)]
    pub enable_discord_presence: bool,
    #[serde(default)]
    pub discord_app_id: String,
    #[serde(default)]
    pub presence_show_artist: bool,
    #[serde(default)]
    pub presence_show_album: bool,
    #[serde(default)]
    pub presence_show_track_duration: bool,
    #[serde(default)]
    pub presence_dynamic_cover: bool,
    #[data(ignore)]
    #[serde(default)]
    pub equalizer: EqualizerConfig,
    #[data(ignore)]
    #[serde(default)]
    pub custom_equalizer_presets: Vec<EqualizerPreset>,
    #[serde(default)]
    pub update_preferences: UpdatePreferences,
    #[data(ignore)]
    #[serde(default)]
    pub keybinds: KeybindsConfig,
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
            enable_discord_presence: false,
            discord_app_id: String::new(),
            presence_show_artist: true,
            presence_show_album: true,
            presence_show_track_duration: true,
            presence_dynamic_cover: false,
            equalizer: Default::default(),
            custom_equalizer_presets: Vec::new(),
            update_preferences: Default::default(),
            keybinds: Default::default(),
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

    #[test]
    fn test_keybinds_default() {
        let keybinds = KeybindsConfig::default();
        assert!(!keybinds.keybinds.is_empty());

        // Check that play/pause keybind exists
        let has_play_pause = keybinds
            .keybinds
            .iter()
            .any(|kb| matches!(kb.action, KeybindAction::PlayPause));
        assert!(has_play_pause);
    }

    #[test]
    fn test_keybinds_serialization() {
        let keybinds = KeybindsConfig::default();
        let json = serde_json::to_string(&keybinds).unwrap();
        let parsed: KeybindsConfig = serde_json::from_str(&json).unwrap();

        // Check that we have the same number of keybinds
        assert_eq!(keybinds.keybinds.len(), parsed.keybinds.len());
    }

    #[test]
    fn test_keybinds_reset_to_defaults() {
        let mut keybinds = KeybindsConfig::default();

        // Clear all keybinds
        keybinds.keybinds.clear();
        assert!(keybinds.keybinds.is_empty());

        // Reset to defaults
        keybinds.reset_to_defaults();
        assert!(!keybinds.keybinds.is_empty());
    }

    #[test]
    fn test_key_combination_display_string() {
        let combo = KeyCombination::new("Space".into(), Some("Space".into()), vec![]);
        assert_eq!(combo.display_string(), "Space");

        let combo_with_ctrl =
            KeyCombination::new("t".into(), Some("KeyT".into()), vec![KeyModifier::Ctrl]);
        assert_eq!(combo_with_ctrl.display_string(), "Ctrl+t");

        let combo_with_multiple = KeyCombination::new(
            "f".into(),
            Some("KeyF".into()),
            vec![KeyModifier::Ctrl, KeyModifier::Shift],
        );
        assert_eq!(combo_with_multiple.display_string(), "Ctrl+Shift+f");
    }

    #[test]
    fn test_keybind_action_display_names() {
        assert_eq!(KeybindAction::PlayPause.display_name(), "Play/Pause");
        assert_eq!(KeybindAction::Next.display_name(), "Next Track");
        assert_eq!(
            KeybindAction::NavigateHome.display_name(),
            "Navigate to Home"
        );
    }
}
