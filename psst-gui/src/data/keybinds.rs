use druid::{Data, KbKey, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Command {
    PlayPause,
    NextTrack,
    PreviousTrack,
    VolumeUp,
    VolumeDown,
    Mute,
    Shuffle,
    Repeat,
    Search,
    GoBack,
    GoHome,
    ShowLyrics,
    CloseWindow,
    Quit,
    Preferences,
}

impl Command {
    pub fn all() -> Vec<Command> {
        vec![
            Command::PlayPause,
            Command::NextTrack,
            Command::PreviousTrack,
            Command::VolumeUp,
            Command::VolumeDown,
            Command::Mute,
            Command::Shuffle,
            Command::Repeat,
            Command::Search,
            Command::GoBack,
            Command::GoHome,
            Command::ShowLyrics,
            Command::CloseWindow,
            Command::Quit,
            Command::Preferences,
        ]
    }

    pub fn to_string(&self) -> String {
        match self {
            Command::PlayPause => "Play / Pause".to_string(),
            Command::NextTrack => "Next Track".to_string(),
            Command::PreviousTrack => "Previous Track".to_string(),
            Command::VolumeUp => "Volume Up".to_string(),
            Command::VolumeDown => "Volume Down".to_string(),
            Command::Mute => "Mute".to_string(),
            Command::Shuffle => "Shuffle".to_string(),
            Command::Repeat => "Repeat".to_string(),
            Command::Search => "Search".to_string(),
            Command::GoBack => "Go Back".to_string(),
            Command::GoHome => "Go Home".to_string(),
            Command::ShowLyrics => "Show Lyrics".to_string(),
            Command::CloseWindow => "Close Window".to_string(),
            Command::Quit => "Quit".to_string(),
            Command::Preferences => "Preferences".to_string(),
        }
    }
}

#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyCombination {
    pub key: String, // Using String to represent KbKey for serialization
    pub mods: u64,   // Bitmask for modifiers
}

impl KeyCombination {
    pub fn new(key: &str, mods: Modifiers) -> Self {
        Self {
            key: key.to_string(),
            mods: mods.bits(),
        }
    }

    pub fn matches(&self, key: &KbKey, mods: &Modifiers) -> bool {
        // Simple string matching for now, can be improved
        let key_str = match key {
            KbKey::Character(c) => c.to_uppercase(),
            KbKey::Enter => "Enter".to_string(),
            KbKey::Escape => "Escape".to_string(),
            KbKey::Backspace => "Backspace".to_string(),
            KbKey::Tab => "Tab".to_string(),
            KbKey::ArrowUp => "ArrowUp".to_string(),
            KbKey::ArrowDown => "ArrowDown".to_string(),
            KbKey::ArrowLeft => "ArrowLeft".to_string(),
            KbKey::ArrowRight => "ArrowRight".to_string(),
            KbKey::F1 => "F1".to_string(),
            KbKey::F2 => "F2".to_string(),
            KbKey::F3 => "F3".to_string(),
            KbKey::F4 => "F4".to_string(),
            KbKey::F5 => "F5".to_string(),
            KbKey::F6 => "F6".to_string(),
            KbKey::F7 => "F7".to_string(),
            KbKey::F8 => "F8".to_string(),
            KbKey::F9 => "F9".to_string(),
            KbKey::F10 => "F10".to_string(),
            KbKey::F11 => "F11".to_string(),
            KbKey::F12 => "F12".to_string(),
            KbKey::Space => "Space".to_string(),
            _ => format!("{:?}", key),
        };

        key_str.eq_ignore_ascii_case(&self.key) && self.mods == mods.bits()
    }

    pub fn to_display_string(&self) -> String {
        let mut s = String::new();
        let mods = Modifiers::from_bits_truncate(self.mods);
        if mods.contains(Modifiers::META) {
            s.push_str("Cmd+");
        }
        if mods.contains(Modifiers::CONTROL) {
            s.push_str("Ctrl+");
        }
        if mods.contains(Modifiers::ALT) {
            s.push_str("Alt+");
        }
        if mods.contains(Modifiers::SHIFT) {
            s.push_str("Shift+");
        }
        s.push_str(&self.key);
        s
    }
}

#[derive(Clone, Debug, Data, Serialize, Deserialize)]
pub struct Keybinds {
    #[data(ignore)]
    pub bindings: HashMap<Command, KeyCombination>,
}

impl Default for Keybinds {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        
        // Default keybinds
        bindings.insert(Command::PlayPause, KeyCombination::new("Space", Modifiers::empty()));
        bindings.insert(Command::NextTrack, KeyCombination::new("ArrowRight", Modifiers::META));
        bindings.insert(Command::PreviousTrack, KeyCombination::new("ArrowLeft", Modifiers::META));
        bindings.insert(Command::Search, KeyCombination::new("L", Modifiers::META));
        bindings.insert(Command::GoBack, KeyCombination::new("ArrowLeft", Modifiers::META | Modifiers::ALT));
        bindings.insert(Command::Preferences, KeyCombination::new(",", Modifiers::META));
        bindings.insert(Command::Quit, KeyCombination::new("Q", Modifiers::META));
        bindings.insert(Command::CloseWindow, KeyCombination::new("W", Modifiers::META));

        Self { bindings }
    }
}

impl Keybinds {
    pub fn get(&self, command: &Command) -> Option<&KeyCombination> {
        self.bindings.get(command)
    }

    pub fn set(&mut self, command: Command, combination: KeyCombination) {
        self.bindings.insert(command, combination);
    }
    
    pub fn find_command(&self, key: &KbKey, mods: &Modifiers) -> Option<Command> {
        for (cmd, combination) in &self.bindings {
            if combination.matches(key, mods) {
                return Some(cmd.clone());
            }
        }
        None
    }
}
