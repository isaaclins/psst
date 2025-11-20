use druid::{commands, platform_menus, Env, LocalizedString, Menu, MenuItem, SysMods, WindowId};

use crate::{
    cmd,
    data::config::{KeyCombination, KeybindAction},
    data::AppState,
};

pub fn main_menu(_window: Option<WindowId>, data: &AppState, env: &Env) -> Menu<AppState> {
    if cfg!(target_os = "macos") {
        Menu::empty().entry(mac_app_menu())
    } else {
        Menu::empty()
    }
    .entry(edit_menu())
    .entry(view_menu(data, env))
}

fn mac_app_menu() -> Menu<AppState> {
    // macOS-only commands are deprecated on other systems.
    #[cfg_attr(not(target_os = "macos"), allow(deprecated))]
    Menu::new(LocalizedString::new("macos-menu-application-menu"))
        .entry(platform_menus::mac::application::preferences())
        .separator()
        .entry(
            // TODO:
            //  This is just overriding `platform_menus::mac::application::quit()`
            //  because l10n is a bit stupid now.
            MenuItem::new(LocalizedString::new("macos-menu-quit").with_placeholder("Quit Psst"))
                .command(cmd::QUIT_APP_WITH_SAVE)
                .hotkey(SysMods::Cmd, "q"),
        )
        .entry(
            MenuItem::new(LocalizedString::new("macos-menu-hide").with_placeholder("Hide Psst"))
                .command(commands::HIDE_APPLICATION)
                .hotkey(SysMods::Cmd, "h"),
        )
        .entry(
            MenuItem::new(
                LocalizedString::new("macos-menu-hide-others").with_placeholder("Hide Others"),
            )
            .command(commands::HIDE_OTHERS)
            .hotkey(SysMods::AltCmd, "h"),
        )
}

fn edit_menu() -> Menu<AppState> {
    Menu::new(LocalizedString::new("common-menu-edit-menu").with_placeholder("Edit"))
        .entry(platform_menus::common::cut())
        .entry(platform_menus::common::copy())
        .entry(platform_menus::common::paste())
}

fn view_menu(data: &AppState, _env: &Env) -> Menu<AppState> {
    let mut menu = Menu::new(LocalizedString::new("menu-view-menu").with_placeholder("View"))
        .rebuild_on(|old_data: &AppState, new_data: &AppState, _env| {
            old_data.preferences.keybind_menu_revision != new_data.preferences.keybind_menu_revision
        });

    const PLAYBACK_ACTIONS: &[KeybindAction] = &[
        KeybindAction::PlayPause,
        KeybindAction::Play,
        KeybindAction::Pause,
        KeybindAction::Stop,
        KeybindAction::Next,
        KeybindAction::Previous,
        KeybindAction::SeekForward,
        KeybindAction::SeekBackward,
        KeybindAction::VolumeUp,
        KeybindAction::VolumeDown,
    ];

    const NAVIGATION_ACTIONS: &[KeybindAction] = &[
        KeybindAction::NavigateHome,
        KeybindAction::NavigateSavedTracks,
        KeybindAction::NavigateSavedAlbums,
        KeybindAction::NavigateShows,
        KeybindAction::NavigateSearch,
        KeybindAction::NavigateBack,
        KeybindAction::NavigateRefresh,
    ];

    const UI_ACTIONS: &[KeybindAction] = &[
        KeybindAction::ToggleSidebar,
        KeybindAction::ToggleLyrics,
        KeybindAction::OpenPreferences,
        KeybindAction::CloseWindow,
        KeybindAction::ToggleFinder,
        KeybindAction::FocusSearch,
    ];

    const QUEUE_ACTIONS: &[KeybindAction] = &[
        KeybindAction::QueueBehaviorSequential,
        KeybindAction::QueueBehaviorRandom,
        KeybindAction::QueueBehaviorLoopTrack,
        KeybindAction::QueueBehaviorLoopAll,
    ];

    let sections = [
        ("Playback Controls", &PLAYBACK_ACTIONS[..]),
        ("Navigation", &NAVIGATION_ACTIONS[..]),
        ("UI Controls", &UI_ACTIONS[..]),
        ("Queue Controls", &QUEUE_ACTIONS[..]),
    ];

    let mut first_section = true;
    for (label, actions) in sections {
        let available_actions: Vec<KeybindAction> = actions
            .iter()
            .copied()
            .filter(|action| data.config.keybinds.get_keybind(*action).is_some())
            .collect();

        if available_actions.is_empty() {
            continue;
        }

        if !first_section {
            menu = menu.separator();
        }

        first_section = false;
        menu = menu.entry(MenuItem::new(label.to_string()).enabled(false));

        for action in available_actions {
            menu = menu.entry(keybind_menu_item(action));
        }
    }

    if first_section {
        menu = menu.entry(MenuItem::new("No keybinds configured").enabled(false));
    }

    menu
}

fn keybind_menu_item(action: KeybindAction) -> MenuItem<AppState> {
    let action_label = action.display_name().to_string();
    MenuItem::new(action_label)
        .command(cmd::PERFORM_KEYBIND_ACTION.with(action))
        .dynamic_hotkey(move |data: &AppState, _env| {
            data.config
                .keybinds
                .get_keybind(action)
                .and_then(KeyCombination::to_hot_key)
        })
}
