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
use std::{env, fmt, io, io::BufRead, path::PathBuf, thread};

const TEST_MODE_ENV: &str = "PSST_CLI_TEST_MODE";

fn main() {
    env_logger::init();

    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let mut args = env::args();
    let _binary = args.next();

    let track_id = args
        .next()
        .ok_or(CliError::MissingTrackId)?;
    let eq_preset_name = args.next();

    let username = env::var("SPOTIFY_USERNAME").map_err(|_| CliError::MissingUsername)?;
    let password = env::var("SPOTIFY_PASSWORD").map_err(|_| CliError::MissingPassword)?;

    let item_id = ItemId::from_base62(&track_id, ItemIdType::Track)
        .ok_or_else(|| CliError::InvalidTrackId(track_id.clone()))?;

    let equalizer = configure_equalizer(eq_preset_name.as_deref());
    let login_creds = Credentials::from_username_and_password(username, password);

    let session = SessionService::with_config(SessionConfig {
        login_creds,
        proxy_url: None,
    });

    if env::var_os(TEST_MODE_ENV).is_some() {
        return Ok(());
    }

    let playback_item = PlaybackItem {
        item_id,
        norm_level: NormalizationLevel::Track,
    };

    start(playback_item, session, equalizer).map_err(CliError::Core)
}

fn configure_equalizer(preset: Option<&str>) -> EqualizerConfig {
    let mut equalizer = EqualizerConfig::default();

    if let Some(preset_name) = preset {
        let presets = EqualizerPreset::built_in_presets();
        if let Some(preset) = presets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(preset_name))
        {
            equalizer.bands = preset.bands.clone();
            equalizer.enabled = true;
            log::info!("Using equalizer preset: {}", preset.name);
        } else {
            log::warn!(
                "Unknown preset '{}', available presets: {}",
                preset_name,
                presets
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    equalizer
}

fn start(
    playback_item: PlaybackItem,
    session: SessionService,
    equalizer: EqualizerConfig,
) -> Result<(), Error> {
    let cdn = Cdn::new(session.clone(), None)?;
    let cache = Cache::new(PathBuf::from("cache"))?;

    play_item(session, cdn, cache, playback_item, equalizer)
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

#[derive(Debug)]
enum CliError {
    MissingTrackId,
    MissingUsername,
    MissingPassword,
    InvalidTrackId(String),
    Core(Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingTrackId => write!(f, "Expected <track_id> in the first parameter"),
            CliError::MissingUsername => {
                write!(f, "Environment variable SPOTIFY_USERNAME is required")
            }
            CliError::MissingPassword => {
                write!(f, "Environment variable SPOTIFY_PASSWORD is required")
            }
            CliError::InvalidTrackId(track) => {
                write!(f, "Invalid Spotify track id: '{track}'")
            }
            CliError::Core(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Core(err) => Some(err),
            _ => None,
        }
    }
}
