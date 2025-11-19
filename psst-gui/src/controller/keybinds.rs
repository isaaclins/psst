use druid::widget::{prelude::*, Controller};

use crate::{
    cmd,
    data::{config::KeybindAction, AppState, Nav, QueueBehavior},
};

/// Controller that handles global keybinds
pub struct KeybindsController;

impl KeybindsController {
    pub fn new() -> Self {
        Self
    }

    fn handle_keybind_action(&self, ctx: &mut EventCtx, action: KeybindAction, data: &mut AppState) {
        match action {
            // Playback controls
            KeybindAction::PlayPause => {
                ctx.submit_command(cmd::PLAY_PAUSE);
            }
            KeybindAction::Play => {
                ctx.submit_command(cmd::PLAY_RESUME);
            }
            KeybindAction::Pause => {
                ctx.submit_command(cmd::PLAY_PAUSE);
            }
            KeybindAction::Next => {
                ctx.submit_command(cmd::PLAY_NEXT);
            }
            KeybindAction::Previous => {
                ctx.submit_command(cmd::PLAY_PREVIOUS);
            }
            KeybindAction::SeekForward => {
                // Seeking is handled by PlaybackController
            }
            KeybindAction::SeekBackward => {
                // Seeking is handled by PlaybackController
            }
            KeybindAction::VolumeUp => {
                data.playback.volume = (data.playback.volume + 0.1).min(1.0);
            }
            KeybindAction::VolumeDown => {
                data.playback.volume = (data.playback.volume - 0.1).max(0.0);
            }
            KeybindAction::Stop => {
                ctx.submit_command(cmd::PLAY_STOP);
            }

            // Navigation
            KeybindAction::NavigateHome => {
                ctx.submit_command(cmd::NAVIGATE.with(Nav::Home));
            }
            KeybindAction::NavigateSavedTracks => {
                ctx.submit_command(cmd::NAVIGATE.with(Nav::SavedTracks));
            }
            KeybindAction::NavigateSavedAlbums => {
                ctx.submit_command(cmd::NAVIGATE.with(Nav::SavedAlbums));
            }
            KeybindAction::NavigateShows => {
                ctx.submit_command(cmd::NAVIGATE.with(Nav::Shows));
            }
            KeybindAction::NavigateSearch => {
                ctx.submit_command(cmd::SET_FOCUS.to(cmd::WIDGET_SEARCH_INPUT));
            }
            KeybindAction::NavigateBack => {
                ctx.submit_command(cmd::NAVIGATE_BACK.with(1));
            }
            KeybindAction::NavigateRefresh => {
                ctx.submit_command(cmd::NAVIGATE_REFRESH);
            }

            // UI Controls
            KeybindAction::ToggleSidebar => {
                data.config.sidebar_visible = !data.config.sidebar_visible;
                data.config.save();
            }
            KeybindAction::ToggleLyrics => {
                ctx.submit_command(cmd::TOGGLE_LYRICS);
            }
            KeybindAction::OpenPreferences => {
                // This is handled at the delegate level usually
            }
            KeybindAction::CloseWindow => {
                ctx.submit_command(cmd::CLOSE_ALL_WINDOWS);
            }
            KeybindAction::ToggleFinder => {
                ctx.submit_command(cmd::TOGGLE_FINDER);
            }
            KeybindAction::FocusSearch => {
                ctx.submit_command(cmd::SET_FOCUS.to(cmd::WIDGET_SEARCH_INPUT));
            }

            // Queue controls
            KeybindAction::QueueBehaviorSequential => {
                ctx.submit_command(cmd::PLAY_QUEUE_BEHAVIOR.with(QueueBehavior::Sequential));
            }
            KeybindAction::QueueBehaviorRandom => {
                ctx.submit_command(cmd::PLAY_QUEUE_BEHAVIOR.with(QueueBehavior::Random));
            }
            KeybindAction::QueueBehaviorLoopTrack => {
                ctx.submit_command(cmd::PLAY_QUEUE_BEHAVIOR.with(QueueBehavior::LoopTrack));
            }
            KeybindAction::QueueBehaviorLoopAll => {
                ctx.submit_command(cmd::PLAY_QUEUE_BEHAVIOR.with(QueueBehavior::LoopAll));
            }
        }
    }
}

impl<W> Controller<AppState, W> for KeybindsController
where
    W: Widget<AppState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(key_event) => {
                // Check if this key event matches any configured keybind
                if let Some(action) = data.config.keybinds.find_action(key_event) {
                    // Handle certain actions that should not override default behavior
                    let should_handle = match action {
                        // Don't override Space and arrow keys if they're already being handled
                        // by PlaybackController
                        KeybindAction::PlayPause
                        | KeybindAction::SeekForward
                        | KeybindAction::SeekBackward
                        | KeybindAction::Next
                        | KeybindAction::Previous => false,
                        _ => true,
                    };

                    if should_handle {
                        self.handle_keybind_action(ctx, action, data);
                        ctx.set_handled();
                        return;
                    }
                }
            }
            _ => {}
        }

        child.event(ctx, event, data, env);
    }
}
