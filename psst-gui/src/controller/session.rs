use druid::widget::{prelude::*, Controller};

use crate::{
    cmd,
    data::AppState,
    ui::{home, playlist, user},
};

pub struct SessionController;

impl SessionController {
    fn connect(&self, ctx: &mut EventCtx, data: &mut AppState) {
        // Update the session configuration, any active session will get shut down.
        data.session.update_config(data.config.session());

        // Re-apply persisted OAuth bearer to both core session and Web API, if present.
        let oauth_bearer = if let Some(profile) = data.config.profile_manager.active_profile() {
            profile.oauth_bearer.clone()
        } else {
            data.config.oauth_bearer.clone()
        };
        
        if let Some(tok) = oauth_bearer {
            data.session.set_oauth_bearer(Some(tok.clone()));
            crate::webapi::WebApi::global().set_oauth_bearer(Some(tok));
        }

        // Reload the global, usually visible data.
        ctx.submit_command(playlist::LOAD_LIST);
        ctx.submit_command(home::LOAD_MADE_FOR_YOU);
        ctx.submit_command(user::LOAD_PROFILE);
    }

    fn switch_profile(&self, ctx: &mut EventCtx, data: &mut AppState, profile_id: std::sync::Arc<str>) {
        // Save current playback state
        let was_playing = matches!(data.playback.state, crate::data::PlaybackState::Playing);
        
        // Stop current playback
        if was_playing {
            ctx.submit_command(cmd::PLAY_STOP);
        }
        
        // Shut down current session
        data.session.shutdown();
        
        // Switch to new profile
        data.config.profile_manager.set_active_profile(profile_id);
        data.config.save();
        data.config.profile_manager.save();
        
        // Reconnect with new profile
        if data.config.has_credentials() {
            self.connect(ctx, data);
            data.info_alert("Switched to new profile");
        } else {
            data.info_alert("Switched to new profile. Please log in.");
        }
    }
}

impl<W> Controller<AppState, W> for SessionController
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
            Event::Command(cmd) if cmd.is(cmd::SESSION_CONNECT) => {
                if data.config.has_credentials() {
                    self.connect(ctx, data);
                }
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(cmd::SWITCH_PROFILE) => {
                let profile_id = cmd.get_unchecked(cmd::SWITCH_PROFILE).clone();
                self.switch_profile(ctx, data, profile_id);
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(cmd::DELETE_PROFILE) => {
                let profile_id = cmd.get_unchecked(cmd::DELETE_PROFILE);
                
                // Don't delete if it's the active profile
                if let Some(active_id) = &data.config.profile_manager.active_profile_id {
                    if active_id.as_ref() == profile_id.as_ref() {
                        data.error_alert("Cannot delete active profile. Switch to another profile first.");
                        ctx.set_handled();
                        return;
                    }
                }
                
                data.config.profile_manager.remove_profile(profile_id.as_ref());
                data.config.profile_manager.save();
                data.info_alert("Profile deleted");
                ctx.set_handled();
            }
            _ => {
                child.event(ctx, event, data, env);
            }
        }
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            ctx.submit_command(cmd::SESSION_CONNECT);
        }
        child.lifecycle(ctx, event, data, env)
    }
}
