use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    cmd,
    data::{
        AppState, AudioQuality, Authentication, Config, CustomTheme, Preferences, PreferencesTab,
        Promise, SliderScrollScale, Theme,
    },
    widget::{icons, Async, Border, Checkbox, MyWidgetExt},
};
use druid::{
    text::ParseFormatter,
    widget::{
        Button, Controller, CrossAxisAlignment, Flex, Label, LineBreaking, MainAxisAlignment,
        Painter, RadioGroup, Scroll, SizedBox, Slider, TextBox, ViewSwitcher,
    },
    Color, Data, Env, Event, EventCtx, Insets, Lens, LensExt, LifeCycle, LifeCycleCtx,
    RenderContext, Selector, Target, Widget, WidgetExt,
};
use psst_core::{connection::Credentials, lastfm, oauth, session::SessionConfig};

use super::{icons::SvgIcon, theme};

const CLEAR_CACHE: Selector = Selector::new("app.preferences.clear-cache");

// Helper function for creating a labeled input row
fn make_input_row<L>(
    label_text: &'static str,
    placeholder_text: &'static str,
    lens: L,
) -> impl Widget<AppState>
where
    L: Lens<AppState, String> + 'static,
{
    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(
            SizedBox::new(Label::new(label_text))
                .width(theme::grid(12.0))
                .align_left(),
        )
        .with_flex_child(
            TextBox::new()
                .with_placeholder(placeholder_text)
                .lens(lens)
                .fix_width(theme::grid(30.0)),
            1.0,
        )
}

pub fn account_setup_widget() -> impl Widget<AppState> {
    Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_spacer(theme::grid(2.0))
        .with_child(
            Label::new("Please insert your Spotify Premium credentials.")
                .with_font(theme::UI_FONT_MEDIUM)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0))
        .with_child(
            Label::new(
                "Psst connects only to the official servers, and does not store your password.",
            )
            .with_text_color(theme::PLACEHOLDER_COLOR)
            .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(6.0))
        .with_child(account_tab_widget(AccountTab::FirstSetup).expand_width())
        .padding(theme::grid(4.0))
}

pub fn preferences_widget() -> impl Widget<AppState> {
    const PROPAGATE_FLAGS: Selector = Selector::new("app.preferences.propagate-flags");

    Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
        .with_child(
            Scroll::new(tabs_widget().padding(theme::grid(2.0)))
                .horizontal()
                .content_must_fill(false)
                .background(theme::BACKGROUND_LIGHT),
        )
        .with_child(
            ViewSwitcher::new(
                |state: &AppState, _| state.preferences.active,
                |active, _, _| match active {
                    PreferencesTab::General => general_tab_widget().boxed(),
                    PreferencesTab::Appearance => appearance_tab_widget().boxed(),
                    PreferencesTab::Equalizer => equalizer_tab_widget().boxed(),
                    PreferencesTab::Account => {
                        account_tab_widget(AccountTab::InPreferences).boxed()
                    }
                    PreferencesTab::Privacy => privacy_tab_widget().boxed(),
                    PreferencesTab::Cache => cache_tab_widget().boxed(),
                    PreferencesTab::About => about_tab_widget().boxed(),
                },
            )
            .padding(theme::grid(4.0))
            .background(Border::Top.with_color(theme::GREY_500)),
        )
        .on_update(|ctx, old_data, data, _| {
            // Immediately save any changes in the config.
            if !old_data.config.same(&data.config) {
                data.config.save();
            }

            // Propagate some flags further to the state.
            let track_cover_changed =
                old_data.config.show_track_cover != data.config.show_track_cover;
            let playlist_covers_changed =
                old_data.config.show_playlist_images != data.config.show_playlist_images;
            if track_cover_changed || playlist_covers_changed {
                ctx.submit_command(PROPAGATE_FLAGS);
            }
        })
        .on_command(PROPAGATE_FLAGS, |_, (), data| {
            let show_track_cover = data.config.show_track_cover;
            let show_playlist_images = data.config.show_playlist_images;
            let common = data.common_ctx_mut();
            common.show_track_cover = show_track_cover;
            common.show_playlist_images = show_playlist_images;
        })
        .scroll()
        .vertical()
        .content_must_fill(true)
        .padding(if cfg!(target_os = "macos") {
            // Accommodate the window controls on Mac.
            Insets::new(0.0, 24.0, 0.0, 0.0)
        } else {
            Insets::ZERO
        })
}

fn tabs_widget() -> impl Widget<AppState> {
    Flex::row()
        .must_fill_main_axis(true)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(tab_link_widget(
            "General",
            &icons::PREFERENCES,
            PreferencesTab::General,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "Appearance",
            &icons::PLAYLIST,
            PreferencesTab::Appearance,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "Equalizer",
            &icons::MUSIC_NOTE,
            PreferencesTab::Equalizer,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "Account",
            &icons::ACCOUNT,
            PreferencesTab::Account,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "Privacy",
            &icons::PREFERENCES,
            PreferencesTab::Privacy,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "Cache",
            &icons::STORAGE,
            PreferencesTab::Cache,
        ))
        .with_default_spacer()
        .with_child(tab_link_widget(
            "About",
            &icons::HEART,
            PreferencesTab::About,
        ))
}

fn tab_link_widget(
    text: &'static str,
    icon: &SvgIcon,
    tab: PreferencesTab,
) -> impl Widget<AppState> {
    Flex::column()
        .with_child(icon.scale(theme::ICON_SIZE_LARGE))
        .with_default_spacer()
        .with_child(Label::new(text).with_font(theme::UI_FONT_MEDIUM))
        .padding(theme::grid(1.0))
        .link()
        .rounded(theme::BUTTON_BORDER_RADIUS)
        .active(move |state: &AppState, _| tab == state.preferences.active)
        .on_left_click(move |_, _, state: &mut AppState, _| {
            state.preferences.active = tab;
        })
        .env_scope(|env, _| {
            env.set(theme::LINK_ACTIVE_COLOR, env.get(theme::BACKGROUND_DARK));
        })
}

fn general_tab_widget() -> impl Widget<AppState> {
    let mut col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true);

    // Audio quality
    col = col
        .with_child(Label::new("Audio quality").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            RadioGroup::column(vec![
                ("Low (96kbit)", AudioQuality::Low),
                ("Normal (160kbit)", AudioQuality::Normal),
                ("High (320kbit)", AudioQuality::High),
            ])
            .lens(AppState::config.then(Config::audio_quality)),
        );

    col = col.with_spacer(theme::grid(3.0));

    // Sliders
    col = col
        .with_child(Label::new("Slider Scrolling").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            Flex::row()
                .with_child(
                    SizedBox::new(Label::dynamic(|state: &AppState, _| {
                        format!("{:.1}", state.config.slider_scroll_scale.scale)
                    }))
                    .width(20.0),
                )
                .with_spacer(theme::grid(0.5))
                .with_child(
                    Slider::new().with_range(0.0, 7.0).lens(
                        AppState::config
                            .then(Config::slider_scroll_scale)
                            .then(SliderScrollScale::scale),
                    ),
                )
                .with_spacer(theme::grid(0.5))
                .with_child(Label::new("Sensitivity")),
        );

    col = col.with_spacer(theme::grid(3.0));

    col = col
        .with_child(Label::new("Seek Duration").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            Flex::row()
                .with_child(
                    TextBox::new().with_formatter(ParseFormatter::with_format_fn(
                        |usize: &usize| usize.to_string(),
                    )),
                )
                .lens(AppState::config.then(Config::seek_duration)),
        );

    col = col.with_spacer(theme::grid(3.0));

    col = col
        .with_child(
            Label::new("Max Loaded Tracks (requires restart)").with_font(theme::UI_FONT_MEDIUM),
        )
        .with_spacer(theme::grid(2.0))
        .with_child(
            Flex::row()
                .with_child(
                    TextBox::new().with_formatter(ParseFormatter::with_format_fn(
                        |usize: &usize| usize.to_string(),
                    )),
                )
                .lens(AppState::config.then(Config::paginated_limit)),
        );

    col
}

fn appearance_tab_widget() -> impl Widget<AppState> {
    let mut col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true);

    col = col
        .with_child(Label::new("Theme").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            RadioGroup::column(vec![
                ("Light", Theme::Light),
                ("Dark", Theme::Dark),
                ("Custom", Theme::Custom),
            ])
            .lens(AppState::config.then(Config::theme)),
        );

    col = col.with_spacer(theme::grid(3.0));

    col = col
        .with_child(custom_theme_section())
        .with_spacer(theme::grid(3.0));

    col = col
        .with_child(Label::new("Artwork").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(1.5))
        .with_child(
            Checkbox::new("Show album covers for tracks")
                .lens(AppState::config.then(Config::show_track_cover)),
        )
        .with_spacer(theme::grid(1.0))
        .with_child(
            Checkbox::new("Show playlist cover images in sidebar")
                .lens(AppState::config.then(Config::show_playlist_images)),
        );

    col
}

fn custom_theme_section() -> impl Widget<AppState> {
    ViewSwitcher::new(
        |state: &AppState, _| state.config.theme,
        |theme, _, _| match theme {
            Theme::Custom => custom_theme_editor().boxed(),
            _ => Label::new("Switch to the Custom theme to edit colors.")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap)
                .boxed(),
        },
    )
    .padding((0.0, theme::grid(1.0), 0.0, 0.0))
}

fn custom_theme_editor() -> impl Widget<AppState> {
    let mut col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(false);

    col = col
        .with_child(Label::new("Custom Colors").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(1.0))
        .with_child(
            Label::new("Update the palette below to restyle the app instantly.")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0));

    col = col
        .with_child(custom_color_row(
            "Color 1",
            "Background — main window backdrop",
            "#1c1c1f",
            druid::KeyOrValue::Key(theme::CUSTOM_COLOR_3),
            AppState::config
                .then(Config::custom_theme)
                .then(CustomTheme::background),
        ))
        .with_child(custom_color_row(
            "Color 2",
            "Surface — cards, sidebars and panels",
            "#242429",
            druid::KeyOrValue::Key(theme::CUSTOM_COLOR_2),
            AppState::config
                .then(Config::custom_theme)
                .then(CustomTheme::surface),
        ))
        .with_child(custom_color_row(
            "Color 3",
            "Primary text — titles and menu items",
            "#f2f2f2",
            druid::KeyOrValue::Key(theme::CUSTOM_COLOR_1),
            AppState::config
                .then(Config::custom_theme)
                .then(CustomTheme::primary_text),
        ))
        .with_child(custom_color_row(
            "Color 4",
            "Accent — buttons and selected chips",
            "#1db954",
            druid::KeyOrValue::Key(theme::CUSTOM_COLOR_4),
            AppState::config
                .then(Config::custom_theme)
                .then(CustomTheme::accent),
        ))
        .with_child(custom_color_row(
            "Color 5",
            "Highlight — progress bar and active tabs",
            "#3a7bd5",
            druid::KeyOrValue::Key(theme::CUSTOM_COLOR_5),
            AppState::config
                .then(Config::custom_theme)
                .then(CustomTheme::highlight),
        ));

    // Typography section
    col = col
        .with_spacer(theme::grid(3.0))
        .with_child(Label::new("Typography").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(1.0))
        .with_child(
            Label::new("Customize fonts (requires restart to take full effect).")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0))
        .with_child(typography_row(
            "Font Family",
            "System UI",
            AppState::config
                .then(Config::custom_theme)
                .then(CustomTheme::font_family),
        ))
        .with_child(typography_row(
            "Font Size",
            "13.0",
            AppState::config
                .then(Config::custom_theme)
                .then(CustomTheme::font_size),
        ));

    // Export/Import section
    col = col
        .with_spacer(theme::grid(3.0))
        .with_child(Label::new("Import/Export").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(1.0))
        .with_child(
            Label::new("Save your custom theme or load a theme file.")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0))
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("Export Theme").on_click(|ctx, data: &mut AppState, _| {
                        export_theme(ctx, data);
                    }),
                )
                .with_spacer(theme::grid(1.0))
                .with_child(
                    Button::new("Import Theme").on_click(|ctx, data: &mut AppState, _| {
                        import_theme(ctx, data);
                    }),
                ),
        );

    col
}

fn typography_row<L>(
    label_text: &'static str,
    placeholder_text: &'static str,
    lens: L,
) -> impl Widget<AppState>
where
    L: Lens<AppState, String> + 'static,
{
    let description = match label_text {
        "Font Family" => {
            "Use 'System UI', 'Serif', 'Sans-Serif', 'Monospace', or any installed font name"
        }
        "Font Size" => "Recommended: 10.0 - 18.0",
        _ => "",
    };

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new(label_text).with_font(theme::UI_FONT_MEDIUM))
        .with_child(
            Label::new(description)
                .with_text_size(theme::TEXT_SIZE_SMALL)
                .with_text_color(theme::PLACEHOLDER_COLOR),
        )
        .with_spacer(theme::grid(0.5))
        .with_child(
            TextBox::new()
                .with_placeholder(placeholder_text)
                .fix_width(theme::grid(30.0))
                .lens(lens),
        )
        .with_spacer(theme::grid(1.5))
}

fn export_theme(ctx: &mut EventCtx, _data: &AppState) {
    use druid::FileDialogOptions;

    let options = FileDialogOptions::new()
        .default_name("custom-theme.json")
        .allowed_types(vec![druid::FileSpec::new("JSON Theme File", &["json"])]);

    ctx.submit_command(
        druid::commands::SHOW_SAVE_PANEL
            .with(options)
            .to(druid::Target::Auto),
    );
}

fn import_theme(ctx: &mut EventCtx, _data: &AppState) {
    use druid::FileDialogOptions;

    let options = FileDialogOptions::new()
        .allowed_types(vec![druid::FileSpec::new("JSON Theme File", &["json"])]);

    ctx.submit_command(
        druid::commands::SHOW_OPEN_PANEL
            .with(options)
            .to(druid::Target::Auto),
    );
}

fn custom_color_row<L>(
    title: &'static str,
    description: &'static str,
    placeholder: &'static str,
    preview_color: druid::KeyOrValue<Color>,
    lens: L,
) -> impl Widget<AppState>
where
    L: Lens<AppState, String> + Clone + 'static,
{
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new(title).with_font(theme::UI_FONT_MEDIUM))
        .with_child(
            Label::new(description)
                .with_text_size(theme::TEXT_SIZE_SMALL)
                .with_text_color(theme::PLACEHOLDER_COLOR),
        )
        .with_spacer(theme::grid(0.5))
        .with_child(
            Flex::row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(
                    TextBox::new()
                        .with_placeholder(placeholder)
                        .fix_width(theme::grid(18.0))
                        .lens(lens.clone()),
                )
                .with_spacer(theme::grid(1.0))
                .with_child({
                    let preview_color = preview_color.clone();
                    let lens_for_click = lens.clone();
                    SizedBox::empty()
                        .fix_size(theme::grid(3.0), theme::grid(3.0))
                        .background(Painter::new(move |ctx, _, env| {
                            let color = preview_color.resolve(env);
                            let rect = ctx.size().to_rect();
                            ctx.fill(rect, &color);
                        }))
                        .rounded(theme::BUTTON_BORDER_RADIUS)
                        .border(theme::BORDER_LIGHT, 1.0)
                        .on_left_click(move |ctx, _, data: &mut AppState, _| {
                            let current = lens_for_click.with(data, |value| value.clone());
                            if let Some(chosen) = choose_system_color(&current) {
                                lens_for_click.with_mut(data, |value| *value = chosen);
                                ctx.request_paint();
                            }
                        })
                }),
        )
        .with_spacer(theme::grid(1.5))
}

struct CacheController {
    thread: Option<JoinHandle<()>>,
}

impl CacheController {
    const RESULT: Selector<Option<u64>> = Selector::new("app.preferences.measure-cache-size");

    fn new() -> Self {
        Self { thread: None }
    }

    fn start_measuring(&mut self, sink: druid::ExtEventSink, widget_id: druid::WidgetId) {
        if self.thread.is_some() {
            return;
        }
        let handle = thread::spawn(move || {
            let size = Preferences::measure_cache_usage();
            sink.submit_command(Self::RESULT, size, widget_id).unwrap();
        });
        self.thread.replace(handle);
    }
}

fn choose_system_color(current: &str) -> Option<String> {
    let (default_r, default_g, default_b) = parse_hex_color(current).unwrap_or((0, 0, 0));

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let script = format!(
            "choose color default color {{{}, {}, {}}}",
            u16::from(default_r) * 257,
            u16::from(default_g) * 257,
            u16::from(default_b) * 257
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut parts = stdout
            .trim()
            .split(',')
            .filter_map(|p| p.trim().parse::<u16>().ok());
        let r = parts.next()?;
        let g = parts.next()?;
        let b = parts.next()?;

        Some(format!("#{:02x}{:02x}{:02x}", r / 257, g / 257, b / 257))
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (default_r, default_g, default_b);
        None
    }
}

fn parse_hex_color(input: &str) -> Option<(u8, u8, u8)> {
    let trimmed = input.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

impl<W: Widget<Preferences>> Controller<Preferences, W> for CacheController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Preferences,
        env: &Env,
    ) {
        match &event {
            Event::Command(cmd) if cmd.is(CLEAR_CACHE) => {
                if let Some(cache) = &data.cache {
                    if let Err(err) = cache.clear() {
                        log::error!("Failed to clear cache: {err}");
                    } else {
                        // After clearing, re-measure the cache size.
                        self.start_measuring(ctx.get_external_handle(), ctx.widget_id());
                    }
                }
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(Self::RESULT) => {
                let result = cmd.get_unchecked(Self::RESULT).to_owned();
                data.cache_size.resolve_or_reject((), result.ok_or(()));
                self.thread.take();
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
        data: &Preferences,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = &event {
            self.start_measuring(ctx.get_external_handle(), ctx.widget_id());
        }
        child.lifecycle(ctx, event, data, env);
    }
}

#[derive(Copy, Clone)]
enum AccountTab {
    FirstSetup,
    InPreferences,
}

fn account_tab_widget(tab: AccountTab) -> impl Widget<AppState> {
    let mut col = Flex::column().cross_axis_alignment(match tab {
        AccountTab::FirstSetup => CrossAxisAlignment::Center,
        AccountTab::InPreferences => CrossAxisAlignment::Start,
    });

    if matches!(tab, AccountTab::InPreferences) {
        col = col
            .with_child(Label::new("Spotify Account").with_font(theme::UI_FONT_MEDIUM))
            .with_spacer(theme::grid(2.0));
    }

    // Spotify Login/Logout button
    col = col
        .with_child(ViewSwitcher::new(
            |data: &AppState, _| data.config.has_credentials(),
            |is_logged_in, _, _| {
                if *is_logged_in {
                    Button::new("Log Out")
                        .on_left_click(|ctx, _, _, _| {
                            ctx.submit_command(cmd::LOG_OUT);
                        })
                        .boxed()
                } else {
                    Button::new("Log in with Spotify")
                        .on_click(|ctx, _data: &mut AppState, _| {
                            ctx.submit_command(Authenticate::SPOTIFY_REQUEST);
                        })
                        .boxed()
                }
            },
        ))
        .with_spacer(theme::grid(1.0))
        .with_child(
            Async::new(
                || Label::new("Logging in...").with_text_size(theme::TEXT_SIZE_SMALL),
                // Spotify Success Arm: Show nothing
                || SizedBox::empty().boxed(),
                || {
                    // Error arm remains the same
                    Label::dynamic(|err: &String, _| err.to_owned())
                        .with_text_size(theme::TEXT_SIZE_SMALL)
                        .with_text_color(druid::Color::RED)
                },
            )
            .lens(
                AppState::preferences
                    .then(Preferences::auth)
                    .then(Authentication::result),
            ),
        );

    if matches!(tab, AccountTab::InPreferences) {
        col = col
            .with_spacer(theme::grid(2.0))
            .with_child(Label::new("Last.fm Account").with_font(theme::UI_FONT_MEDIUM))
            .with_spacer(theme::grid(1.0))
            .with_child(
                Label::new("Connect your Last.fm account to scrobble tracks you listen to.")
                    .with_text_color(theme::PLACEHOLDER_COLOR)
                    .with_line_break_mode(LineBreaking::WordWrap),
            )
            .with_spacer(theme::grid(2.0))
            .with_child(ViewSwitcher::new(
                |data: &AppState, _| data.config.lastfm_session_key.is_some(),
                |connected, _, _| {
                    if *connected {
                        // --- Connected View ---
                        lastfm_connected_view().boxed()
                    } else {
                        // --- Disconnected View ---
                        lastfm_disconnected_view().boxed()
                    }
                },
            ));
    }
    col.controller(Authenticate::new(tab))
}

fn lastfm_connected_view() -> impl Widget<AppState> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(
            Flex::row()
                .with_child(
                    Checkbox::new("Toggle scrobbling")
                        .lens(AppState::config.then(Config::lastfm_enable))
                        .padding((0.0, 0.0, theme::grid(1.0), 0.0)),
                )
                .with_child(
                    Button::new("Disconnect").on_click(|_ctx, data: &mut AppState, _| {
                        data.config.lastfm_session_key = None;
                        data.config.lastfm_api_key = None;
                        data.config.lastfm_api_secret = None;
                        data.config.save();
                        data.preferences.lastfm_auth_result = None;
                        data.preferences.auth.lastfm_api_key_input.clear();
                        data.preferences.auth.lastfm_api_secret_input.clear();
                    }),
                ),
        )
}

fn lastfm_disconnected_view() -> impl Widget<AppState> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(make_input_row(
            "API Key:",
            "Enter your Last.fm API Key",
            AppState::preferences
                .then(Preferences::auth)
                .then(Authentication::lastfm_api_key_input),
        ))
        .with_default_spacer()
        .with_child(make_input_row(
            "API Secret:",
            "Enter your Last.fm API Secret",
            AppState::preferences
                .then(Preferences::auth)
                .then(Authentication::lastfm_api_secret_input),
        ))
        .with_spacer(theme::grid(2.0))
        .with_child(
            Flex::row() // Put buttons in a row
                .with_child(Button::new("Connect Last.fm Account").on_click(
                    |ctx, data: &mut AppState, _| {
                        // Check temporary input fields before proceeding
                        let key_input = &data.preferences.auth.lastfm_api_key_input;
                        let secret_input = &data.preferences.auth.lastfm_api_secret_input;

                        if key_input.is_empty() || secret_input.is_empty() {
                            data.preferences.lastfm_auth_result =
                                Some("API Key and Secret required.".to_string());
                        } else {
                            ctx.submit_command(Authenticate::LASTFM_REQUEST);
                        }
                    },
                ))
                .with_spacer(theme::grid(1.0))
                .with_child(Button::new("Request API Key").on_click(|_, _, _| {
                    open::that("https://www.last.fm/api/account/create").ok();
                })),
        )
        .with_spacer(theme::grid(1.0))
        // Last.fm Status label
        .with_child(ViewSwitcher::new(
            |data: &AppState, _| {
                data.preferences
                    .lastfm_auth_result
                    .clone()
                    .unwrap_or_default()
            },
            |msg: &String, _, _| {
                // Only show label if there's an error or connecting message
                if msg.is_empty() || msg.starts_with("Success") {
                    SizedBox::empty().boxed()
                } else {
                    Label::new(msg.clone())
                        .with_text_color(if msg.starts_with("Connect") {
                            druid::Color::GRAY
                        } else {
                            druid::Color::RED
                        })
                        .boxed()
                }
            },
        ))
}

pub struct Authenticate {
    tab: AccountTab,
    spotify_thread: Option<JoinHandle<()>>,
    lastfm_thread: Option<JoinHandle<()>>,
}

impl Authenticate {
    fn new(tab: AccountTab) -> Self {
        Self {
            tab,
            spotify_thread: None,
            lastfm_thread: None,
        }
    }

    // Helper function to spawn authentication threads
    fn spawn_auth_thread<T: Send + 'static>(
        ctx: &mut EventCtx,
        auth_logic: impl FnOnce() -> Result<T, String> + Send + 'static,
        response_selector: Selector<Result<T, String>>,
        existing_handle: Option<JoinHandle<()>>,
    ) -> Option<JoinHandle<()>> {
        // Clean up previous thread if any
        if let Some(_handle) = existing_handle {
            // Consider if joining is necessary/desirable
        }

        let window_id = ctx.window_id();
        let event_sink = ctx.get_external_handle();

        let thread = thread::spawn(move || {
            let result = auth_logic();
            event_sink
                .submit_command(response_selector, result, window_id)
                .unwrap();
        });
        Some(thread)
    }

    // Helper method to simplify Spotify authentication flow
    fn start_spotify_auth(&mut self, ctx: &mut EventCtx, data: &mut AppState) {
        // Set authentication to in-progress state
        data.preferences.auth.result.defer_default();

        // Generate auth URL and store PKCE verifier
        let (auth_url, pkce_verifier) = oauth::generate_auth_url(8888);
        let config = data.preferences.auth.session_config(); // Keep config local

        // Spawn authentication thread
        self.spotify_thread = Authenticate::spawn_auth_thread(
            ctx,
            move || {
                // Listen for authorization code
                let code = oauth::get_authcode_listener(
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888),
                    Duration::from_secs(300),
                )
                .map_err(|e| e.to_string())?;

                // Exchange code for access and refresh tokens
                let (access, refresh) = oauth::exchange_code_for_token(8888, code, pkce_verifier);

                // Try to authenticate with token, with retries
                let mut retries = 3;
                while retries > 0 {
                    match Authentication::authenticate_and_get_credentials(SessionConfig {
                        login_creds: Credentials::from_access_token(access.clone()),
                        ..config.clone()
                    }) {
                        Ok(credentials) => {
                            return Ok((credentials, access.clone(), refresh.clone()))
                        }
                        Err(e) if retries > 1 => {
                            log::warn!("authentication failed, retrying: {e:?}");
                            retries -= 1;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err("Authentication retries exceeded".to_string())
            },
            Self::SPOTIFY_RESPONSE,
            self.spotify_thread.take(),
        );

        // Open browser with authorization URL
        if open::that(&auth_url).is_err() {
            data.error_alert("Failed to open browser");
            // Resolve the promise with an error immediately
            data.preferences
                .auth
                .result
                .reject((), "Failed to open browser".to_string());
        }
    }
}

impl Authenticate {
    pub const SPOTIFY_REQUEST: Selector =
        Selector::new("app.preferences.spotify.authenticate-request");
    pub const SPOTIFY_RESPONSE: Selector<Result<(Credentials, String, Option<String>), String>> =
        Selector::new("app.preferences.spotify.authenticate-response");

    // Selector for initializing fields
    pub const INITIALIZE_LASTFM_FIELDS: Selector =
        Selector::new("app.preferences.lastfm.initialize-fields");

    // Last.fm selectors
    pub const LASTFM_REQUEST: Selector =
        Selector::new("app.preferences.lastfm.authenticate-request");
    pub const LASTFM_RESPONSE: Selector<Result<String, String>> =
        Selector::new("app.preferences.lastfm.authenticate-response");
}

fn logout_and_reset(ctx: &mut EventCtx, data: &mut AppState) {
    data.config.clear_credentials();
    crate::token_utils::TokenUtils::clear_all(&data.session, &mut data.config, true);
    data.session.shutdown();
    ctx.submit_command(cmd::CLOSE_ALL_WINDOWS);
    ctx.submit_command(cmd::SHOW_ACCOUNT_SETUP);
}

impl<W: Widget<AppState>> Controller<AppState, W> for Authenticate {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(Self::SPOTIFY_REQUEST) => {
                self.start_spotify_auth(ctx, data);
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(Self::INITIALIZE_LASTFM_FIELDS) => {
                data.preferences.auth.lastfm_api_key_input =
                    data.config.lastfm_api_key.clone().unwrap_or_default();
                data.preferences.auth.lastfm_api_secret_input =
                    data.config.lastfm_api_secret.clone().unwrap_or_default();
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(cmd::LOG_OUT) => {
                logout_and_reset(ctx, data);
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(Self::LASTFM_REQUEST) => {
                // Use the temporary input fields from preferences state.
                let api_key = data.preferences.auth.lastfm_api_key_input.clone();
                let api_secret = data.preferences.auth.lastfm_api_secret_input.clone();

                if api_key.is_empty() || api_secret.is_empty() {
                    data.preferences.lastfm_auth_result =
                        Some("API Key and Secret required.".to_string());
                    ctx.set_handled();
                    return;
                }

                data.preferences.lastfm_auth_result = Some("Connecting...".to_string());
                let port = 8889;
                let callback_url = format!("http://127.0.0.1:{port}/lastfm_callback");
                let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

                match lastfm::generate_lastfm_auth_url(&api_key, &callback_url) {
                    Ok(auth_url) => {
                        self.lastfm_thread = Authenticate::spawn_auth_thread(
                            ctx,
                            move || {
                                let token = lastfm::get_lastfm_token_listener(
                                    socket_addr,
                                    Duration::from_secs(300),
                                )
                                .map_err(|e| e.to_string())?;
                                log::info!("received Last.fm token, exchanging...");
                                lastfm::exchange_token_for_session(&api_key, &api_secret, &token)
                                    .map_err(|e| format!("Token exchange failed: {e}"))
                            },
                            Self::LASTFM_RESPONSE,
                            self.lastfm_thread.take(),
                        );

                        if open::that(&auth_url).is_err() {
                            data.preferences.lastfm_auth_result =
                                Some("Failed to open browser.".to_string());
                            // No promise to reject here, just update the status message
                        }
                    }
                    Err(e) => {
                        data.preferences.lastfm_auth_result =
                            Some(format!("Failed to create auth URL: {e}"));
                    }
                }
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(Self::SPOTIFY_RESPONSE) => {
                let result = cmd.get_unchecked(Self::SPOTIFY_RESPONSE);
                match result {
                    Ok((credentials, token, refresh)) => {
                        log::info!(
                            "OAuth: Spotify auth response success (access_token_present={}, refresh_token_present={})",
                            !token.is_empty(),
                            refresh.as_ref().is_some()
                        );
                        // Apply and persist tokens atomically (runtime + config) to keep
                        // Session/WebApi in sync.
                        log::info!(
                            "OAuth: installing tokens into Session/WebApi and persisting to Config"
                        );
                        crate::token_utils::TokenUtils::apply_and_persist(
                            &data.session,
                            &mut data.config,
                            Some(token.clone()),
                            refresh.clone(),
                            /* clear_refresh_if_none = */ false,
                            /* save = */ false,
                        );
                        // Update session config with the new credentials
                        log::info!("OAuth: updating session config with new credentials");
                        data.session.update_config(SessionConfig {
                            login_creds: credentials.clone(),
                            proxy_url: Config::proxy(),
                        });
                        data.config.store_credentials(credentials.clone());
                        data.config.save();
                        log::info!("OAuth: config saved and tokens applied");
                        data.preferences.auth.result.resolve((), ());
                        // Handle UI flow based on tab type
                        if matches!(self.tab, AccountTab::FirstSetup) {
                            ctx.submit_command(cmd::CLOSE_ALL_WINDOWS);
                            ctx.submit_command(cmd::SHOW_MAIN);
                        }
                    }
                    Err(err) => {
                        log::warn!("OAuth: Spotify authentication failed: {}", err);
                        data.preferences.auth.result.reject((), err.clone());
                    }
                }
                self.spotify_thread.take();
                ctx.set_handled();
            }
            Event::Command(cmd) if cmd.is(Self::LASTFM_RESPONSE) => {
                let result = cmd.get_unchecked(Self::LASTFM_RESPONSE);
                match result {
                    Ok(session_key) => {
                        // On success, store the validated key/secret in config and save.
                        data.config.lastfm_api_key =
                            Some(data.preferences.auth.lastfm_api_key_input.clone());
                        data.config.lastfm_api_secret =
                            Some(data.preferences.auth.lastfm_api_secret_input.clone());
                        data.config.lastfm_session_key = Some(session_key.clone());
                        data.config.save();

                        log::info!("Last.fm session key stored successfully.");

                        data.preferences.lastfm_auth_result =
                            Some("Success! Last.fm connected.".to_string());
                    }
                    Err(err) => {
                        data.preferences.lastfm_auth_result = Some(err.clone());
                    }
                }
                self.lastfm_thread.take();
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
            ctx.submit_command(Self::INITIALIZE_LASTFM_FIELDS);
        }
        child.lifecycle(ctx, event, data, env);
    }
}

fn privacy_tab_widget() -> impl Widget<AppState> {
    let mut col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true);

    // Discord Rich Presence section
    col = col
        .with_child(Label::new("Social Presence").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            Label::new("Control what information is shared when you're listening to music.")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0));

    col = col
        .with_child(
            Checkbox::new("Enable Discord Rich Presence")
                .lens(AppState::config.then(Config::enable_discord_presence)),
        )
        .with_spacer(theme::grid(2.0));

    // Discord App ID input
    col = col
        .with_child(
            Label::new("Discord Application ID:")
                .with_text_size(theme::TEXT_SIZE_SMALL),
        )
        .with_spacer(theme::grid(0.5))
        .with_child(
            TextBox::new()
                .with_placeholder("Enter your Discord Application ID")
                .lens(AppState::config.then(Config::discord_app_id))
                .fix_width(theme::grid(30.0)),
        )
        .with_spacer(theme::grid(0.5))
        .with_child(
            Label::new("Register an application at discord.com/developers to get an Application ID")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_text_size(theme::TEXT_SIZE_SMALL)
                .with_line_break_mode(LineBreaking::WordWrap),
        );

    col = col.with_spacer(theme::grid(3.0));

    // Privacy controls section
    col = col
        .with_child(Label::new("Presence Information").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            Label::new("Choose what information to display in Discord Rich Presence and macOS Now Playing.")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0));

    col = col
        .with_child(
            Checkbox::new("Show artist name")
                .lens(AppState::config.then(Config::presence_show_artist)),
        )
        .with_spacer(theme::grid(1.0))
        .with_child(
            Checkbox::new("Show album name")
                .lens(AppState::config.then(Config::presence_show_album)),
        )
        .with_spacer(theme::grid(1.0))
        .with_child(
            Checkbox::new("Show track duration")
                .lens(AppState::config.then(Config::presence_show_track_duration)),
        );

    col
}

fn cache_tab_widget() -> impl Widget<AppState> {
    let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);

    col = col
        .with_child(Label::new("Location").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(
            Label::dynamic(|_, _| {
                Config::cache_dir()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_else(|| "None".to_string())
            })
            .with_line_break_mode(LineBreaking::WordWrap),
        );

    col = col.with_spacer(theme::grid(3.0));

    col = col
        .with_child(Label::new("Size").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(Label::dynamic(
            |preferences: &Preferences, _| match preferences.cache_size {
                Promise::Empty | Promise::Rejected { .. } => "Unknown".to_string(),
                Promise::Deferred { .. } => "Computing...".to_string(),
                Promise::Resolved { val: 0, .. } => "Empty".to_string(),
                Promise::Resolved { val, .. } => {
                    format!("{:.2} MB", val as f64 / 1e6_f64)
                }
            },
        ));

    // Clear cache button
    col = col
        .with_spacer(theme::grid(2.0))
        .with_child(Button::new("Clear Cache").on_left_click(|ctx, _, _, _| {
            ctx.submit_command(CLEAR_CACHE);
        }));

    col.controller(CacheController::new())
        .lens(AppState::preferences)
}

fn equalizer_tab_widget() -> impl Widget<AppState> {
    use psst_core::audio::equalizer::EqualizerPreset;

    let mut col = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true);

    // Enable/Disable toggle
    col = col
        .with_child(Label::new("Equalizer").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(Label::new("Enable Equalizer").with_text_color(theme::PLACEHOLDER_COLOR))
        .with_spacer(theme::grid(1.0))
        .with_child(
            Button::new(|data: &AppState, _: &_| {
                if data.config.equalizer.enabled {
                    "Disable Equalizer".to_string()
                } else {
                    "Enable Equalizer".to_string()
                }
            })
            .on_click(|_ctx, data: &mut AppState, _| {
                data.config.equalizer.enabled = !data.config.equalizer.enabled;
                data.config.save();
            }),
        );

    col = col.with_spacer(theme::grid(3.0));

    // Preset selector
    col = col
        .with_child(Label::new("Presets").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(1.0))
        .with_child(
            Label::new("Choose a preset or customize your own equalizer curve below.")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0));

    // Add buttons for each preset
    let presets = EqualizerPreset::built_in_presets();
    let mut preset_row = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);

    for (i, preset) in presets.iter().enumerate() {
        let preset_clone = preset.clone();
        preset_row = preset_row.with_child(
            Button::new(preset.name.clone())
                .on_click(move |_ctx, data: &mut AppState, _| {
                    data.config.equalizer.bands = preset_clone.bands.clone();
                    data.config.save();
                })
                .padding((theme::grid(0.5), theme::grid(0.5))),
        );
        if i < presets.len() - 1 {
            preset_row = preset_row.with_spacer(theme::grid(1.0));
        }
    }
    col = col.with_child(preset_row);

    col = col.with_spacer(theme::grid(3.0));

    // Custom EQ bands
    col = col
        .with_child(Label::new("Custom Equalizer").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(1.0))
        .with_child(
            Label::new("Adjust individual frequency bands (in dB, -12 to +12).")
                .with_text_color(theme::PLACEHOLDER_COLOR)
                .with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_spacer(theme::grid(2.0));

    // Add sliders for each band
    for band_index in 0..10 {
        col = col.with_child(equalizer_band_slider(band_index));
    }

    col.controller(EqualizerConfigNotifier)
}

struct EqualizerConfigNotifier;

impl<W> Controller<AppState, W> for EqualizerConfigNotifier
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
        if let Event::Command(command) = event {
            if command.is(cmd::EQUALIZER_CONFIG_CHANGED) {
                child.event(ctx, event, data, env);
                return;
            }
        }

        let before = data.config.equalizer.clone();
        child.event(ctx, event, data, env);
        if before != data.config.equalizer {
            ctx.submit_command(
                cmd::EQUALIZER_CONFIG_CHANGED
                    .with(data.config.equalizer.clone())
                    .to(Target::Global),
            );
        }
    }
}

// Custom lens for accessing a specific equalizer band's gain
// Druid's Slider uses f64, but our config uses f32, so we need to convert
struct EqualizerBandLens {
    index: usize,
}

impl Lens<AppState, f64> for EqualizerBandLens {
    fn with<V, F: FnOnce(&f64) -> V>(&self, data: &AppState, f: F) -> V {
        if self.index < data.config.equalizer.bands.len() {
            let val = data.config.equalizer.bands[self.index].gain_db as f64;
            f(&val)
        } else {
            f(&0.0)
        }
    }

    fn with_mut<V, F: FnOnce(&mut f64) -> V>(&self, data: &mut AppState, f: F) -> V {
        if self.index < data.config.equalizer.bands.len() {
            let mut val = data.config.equalizer.bands[self.index].gain_db as f64;
            let result = f(&mut val);
            data.config.equalizer.bands[self.index].gain_db = val as f32;
            result
        } else {
            let mut temp = 0.0;
            f(&mut temp)
        }
    }
}

fn equalizer_band_slider(band_index: usize) -> impl Widget<AppState> {
    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(
            Label::dynamic(move |data: &AppState, _| {
                if band_index < data.config.equalizer.bands.len() {
                    let freq = data.config.equalizer.bands[band_index].frequency;
                    if freq >= 1000.0 {
                        format!("{:.1} kHz", freq / 1000.0)
                    } else {
                        format!("{:.0} Hz", freq)
                    }
                } else {
                    String::new()
                }
            })
            .with_text_size(theme::TEXT_SIZE_SMALL)
            .fix_width(theme::grid(7.0))
            .align_right(),
        )
        .with_spacer(theme::grid(1.0))
        .with_flex_child(
            Slider::new()
                .with_range(-12.0, 12.0)
                .lens(EqualizerBandLens { index: band_index }),
            1.0,
        )
        .with_spacer(theme::grid(1.0))
        .with_child(
            Label::dynamic(move |data: &AppState, _| {
                if band_index < data.config.equalizer.bands.len() {
                    format!("{:+.1} dB", data.config.equalizer.bands[band_index].gain_db)
                } else {
                    String::new()
                }
            })
            .with_text_size(theme::TEXT_SIZE_SMALL)
            .fix_width(theme::grid(7.0)),
        )
        .padding((0.0, theme::grid(0.3), 0.0, 0.0))
}

fn about_tab_widget() -> impl Widget<AppState> {
    // Build Info
    let commit_hash = Flex::row()
        .with_child(Label::new("Commit Hash:   "))
        .with_child(Label::new(psst_core::GIT_VERSION).with_text_color(theme::DISABLED_TEXT_COLOR));

    let build_time = Flex::row()
        .with_child(Label::new("Build time:   "))
        .with_child(Label::new(psst_core::BUILD_TIME).with_text_color(theme::DISABLED_TEXT_COLOR));

    let remote_url = Flex::row().with_child(Label::new("Source:   ")).with_child(
        Label::new(psst_core::REMOTE_URL)
            .with_text_color(Color::rgb8(138, 180, 248))
            .on_left_click(|_, _, _, _| {
                open::that(psst_core::REMOTE_URL).ok();
            }),
    );

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true)
        .with_child(Label::new("Build Info").with_font(theme::UI_FONT_MEDIUM))
        .with_spacer(theme::grid(2.0))
        .with_child(commit_hash)
        .with_child(build_time)
        .with_child(remote_url)
}
