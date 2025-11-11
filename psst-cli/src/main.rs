use psst_core::{
    audio::{
        equalizer::{EqualizerConfig, EqualizerPreset},
        normalize::NormalizationLevel,
        output::{AudioOutput, AudioSink, DefaultAudioOutput},
    },
    cache::{Cache, CacheHandle},
    cdn::{Cdn, CdnHandle},
    connection::Credentials,
    error::Error,
    item_id::{ItemId, ItemIdType},
    player::{item::PlaybackItem, PlaybackConfig, Player, PlayerCommand, PlayerEvent},
    session::{SessionConfig, SessionService},
};
use std::{env, io, io::BufRead, path::PathBuf, thread};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let track_id = args
        .get(1)
        .expect("Expected <track_id> in the first parameter");
    
    // Optional: Get equalizer preset from command line (2nd argument)
    let eq_preset_name = args.get(2).map(|s| s.as_str());
    
    let login_creds = Credentials::from_username_and_password(
        env::var("SPOTIFY_USERNAME").unwrap(),
        env::var("SPOTIFY_PASSWORD").unwrap(),
    );
    let session = SessionService::with_config(SessionConfig {
        login_creds,
        proxy_url: None,
    });

    start(track_id, session, eq_preset_name).unwrap();
}

fn start(track_id: &str, session: SessionService, eq_preset_name: Option<&str>) -> Result<(), Error> {
    let cdn = Cdn::new(session.clone(), None)?;
    let cache = Cache::new(PathBuf::from("cache"))?;
    let item_id = ItemId::from_base62(track_id, ItemIdType::Track).unwrap();
    
    // Configure equalizer based on preset name
    let mut equalizer = EqualizerConfig::default();
    if let Some(preset_name) = eq_preset_name {
        let presets = EqualizerPreset::built_in_presets();
        if let Some(preset) = presets.iter().find(|p| p.name.eq_ignore_ascii_case(preset_name)) {
            equalizer.bands = preset.bands.clone();
            equalizer.enabled = true;
            log::info!("Using equalizer preset: {}", preset.name);
        } else {
            log::warn!("Unknown preset '{}', available presets: {}", 
                preset_name,
                presets.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
            );
        }
    }
    
    play_item(
        session,
        cdn,
        cache,
        PlaybackItem {
            item_id,
            norm_level: NormalizationLevel::Track,
        },
        equalizer,
    )
}

fn play_item(
    session: SessionService,
    cdn: CdnHandle,
    cache: CacheHandle,
    item: PlaybackItem,
    equalizer: EqualizerConfig,
) -> Result<(), Error> {
    let output = DefaultAudioOutput::open()?;
    let config = PlaybackConfig {
        equalizer,
        ..PlaybackConfig::default()
    };

    let mut player = Player::new(session, cdn, cache, config, &output);

    let _ui_thread = thread::spawn({
        let player_sender = player.sender();

        player_sender
            .send(PlayerEvent::Command(PlayerCommand::LoadQueue {
                items: vec![item, item, item],
                position: 0,
            }))
            .unwrap();

        move || {
            for line in io::stdin().lock().lines() {
                match line.as_ref().map(|s| s.as_str()) {
                    Ok("p") => {
                        player_sender
                            .send(PlayerEvent::Command(PlayerCommand::Pause))
                            .unwrap();
                    }
                    Ok("r") => {
                        player_sender
                            .send(PlayerEvent::Command(PlayerCommand::Resume))
                            .unwrap();
                    }
                    Ok("s") => {
                        player_sender
                            .send(PlayerEvent::Command(PlayerCommand::Stop))
                            .unwrap();
                    }
                    Ok("<") => {
                        player_sender
                            .send(PlayerEvent::Command(PlayerCommand::Previous))
                            .unwrap();
                    }
                    Ok(">") => {
                        player_sender
                            .send(PlayerEvent::Command(PlayerCommand::Next))
                            .unwrap();
                    }
                    _ => log::warn!("unknown command"),
                }
            }
        }
    });

    for event in player.receiver() {
        player.handle(event);
    }
    output.sink().close();

    Ok(())
}
