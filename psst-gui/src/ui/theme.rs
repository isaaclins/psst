use druid::{Color, Env, FontDescriptor, FontFamily, FontWeight, Insets, Key, Size};

pub use druid::theme::*;

use crate::data::{AppState, CustomTheme, Theme};

pub fn grid(m: f64) -> f64 {
    GRID * m
}

pub const GRID: f64 = 8.0;

pub const GREY_000: Key<Color> = Key::new("app.grey_000");
pub const GREY_100: Key<Color> = Key::new("app.grey_100");
pub const GREY_200: Key<Color> = Key::new("app.grey_200");
pub const GREY_300: Key<Color> = Key::new("app.grey_300");
pub const GREY_400: Key<Color> = Key::new("app.grey_400");
pub const GREY_500: Key<Color> = Key::new("app.grey_500");
pub const GREY_600: Key<Color> = Key::new("app.grey_600");
pub const GREY_700: Key<Color> = Key::new("app.grey_700");
pub const BLUE_100: Key<Color> = Key::new("app.blue_100");
pub const BLUE_200: Key<Color> = Key::new("app.blue_200");

pub const CUSTOM_COLOR_1: Key<Color> = Key::new("app.custom_color_1");
pub const CUSTOM_COLOR_2: Key<Color> = Key::new("app.custom_color_2");
pub const CUSTOM_COLOR_3: Key<Color> = Key::new("app.custom_color_3");
pub const CUSTOM_COLOR_4: Key<Color> = Key::new("app.custom_color_4");
pub const CUSTOM_COLOR_5: Key<Color> = Key::new("app.custom_color_5");

pub const RED: Key<Color> = Key::new("app.red");

pub const MENU_BUTTON_BG_ACTIVE: Key<Color> = Key::new("app.menu-bg-active");
pub const MENU_BUTTON_BG_INACTIVE: Key<Color> = Key::new("app.menu-bg-inactive");
pub const MENU_BUTTON_FG_ACTIVE: Key<Color> = Key::new("app.menu-fg-active");
pub const MENU_BUTTON_FG_INACTIVE: Key<Color> = Key::new("app.menu-fg-inactive");

pub const UI_FONT_MEDIUM: Key<FontDescriptor> = Key::new("app.ui-font-medium");
pub const UI_FONT_MONO: Key<FontDescriptor> = Key::new("app.ui-font-mono");
pub const TEXT_SIZE_SMALL: Key<f64> = Key::new("app.text-size-small");

pub const ICON_COLOR: Key<Color> = Key::new("app.icon-color");
pub const ICON_SIZE_TINY: Size = Size::new(12.0, 12.0);
pub const ICON_SIZE_SMALL: Size = Size::new(14.0, 14.0);
pub const ICON_SIZE_MEDIUM: Size = Size::new(16.0, 16.0);
pub const ICON_SIZE_LARGE: Size = Size::new(22.0, 22.0);

pub const LINK_HOT_COLOR: Key<Color> = Key::new("app.link-hot-color");
pub const LINK_ACTIVE_COLOR: Key<Color> = Key::new("app.link-active-color");
pub const LINK_COLD_COLOR: Key<Color> = Key::new("app.link-cold-color");

pub fn setup(env: &mut Env, state: &AppState) {
    match state.config.theme {
        Theme::Light => setup_light_theme(env),
        Theme::Dark => setup_dark_theme(env),
        Theme::Custom => setup_custom_theme(env, &state.config.custom_theme),
    };

    env.set(WINDOW_BACKGROUND_COLOR, env.get(GREY_700));
    env.set(TEXT_COLOR, env.get(GREY_100));
    env.set(ICON_COLOR, env.get(GREY_400));
    env.set(PLACEHOLDER_COLOR, env.get(GREY_300));
    env.set(PRIMARY_LIGHT, env.get(BLUE_100));
    env.set(PRIMARY_DARK, env.get(BLUE_200));

    env.set(BACKGROUND_LIGHT, env.get(GREY_700));
    env.set(BACKGROUND_DARK, env.get(GREY_600));
    env.set(FOREGROUND_LIGHT, env.get(GREY_100));
    env.set(FOREGROUND_DARK, env.get(GREY_000));

    match state.config.theme {
        Theme::Light => {
            env.set(BUTTON_LIGHT, env.get(GREY_700));
            env.set(BUTTON_DARK, env.get(GREY_600));
        }
        Theme::Dark => {
            env.set(BUTTON_LIGHT, env.get(GREY_600));
            env.set(BUTTON_DARK, env.get(GREY_700));
        }
        Theme::Custom => {
            env.set(BUTTON_LIGHT, env.get(CUSTOM_COLOR_2));
            env.set(BUTTON_DARK, env.get(CUSTOM_COLOR_3));
        }
    }

    env.set(BORDER_LIGHT, env.get(GREY_400));
    env.set(BORDER_DARK, env.get(GREY_500));

    match state.config.theme {
        Theme::Custom => {
            env.set(SELECTED_TEXT_BACKGROUND_COLOR, env.get(CUSTOM_COLOR_4));
            env.set(SELECTION_TEXT_COLOR, env.get(CUSTOM_COLOR_2));
        }
        _ => {
            env.set(SELECTED_TEXT_BACKGROUND_COLOR, env.get(BLUE_200));
            env.set(SELECTION_TEXT_COLOR, env.get(GREY_700));
        }
    }

    env.set(
        CURSOR_COLOR,
        match state.config.theme {
            Theme::Custom => env.get(CUSTOM_COLOR_1),
            _ => env.get(GREY_000),
        },
    );

    env.set(PROGRESS_BAR_RADIUS, 4.0);
    env.set(BUTTON_BORDER_RADIUS, 4.0);
    env.set(BUTTON_BORDER_WIDTH, 1.0);

    // Set fonts based on theme
    let (font_family, font_size) = match state.config.theme {
        Theme::Custom => {
            let family = parse_font_family(&state.config.custom_theme.font_family);
            let size = state.config.custom_theme.font_size.parse::<f64>().unwrap_or(13.0);
            (family, size)
        }
        _ => (FontFamily::SYSTEM_UI, 13.0),
    };

    env.set(
        UI_FONT,
        FontDescriptor::new(font_family.clone()).with_size(font_size),
    );
    env.set(
        UI_FONT_MEDIUM,
        FontDescriptor::new(font_family.clone())
            .with_size(font_size)
            .with_weight(FontWeight::MEDIUM),
    );
    env.set(
        UI_FONT_MONO,
        FontDescriptor::new(FontFamily::MONOSPACE).with_size(font_size),
    );
    env.set(TEXT_SIZE_SMALL, font_size - 2.0);
    env.set(TEXT_SIZE_NORMAL, font_size);
    env.set(TEXT_SIZE_LARGE, font_size + 3.0);

    env.set(BASIC_WIDGET_HEIGHT, 16.0);
    env.set(WIDE_WIDGET_WIDTH, grid(12.0));
    env.set(BORDERED_WIDGET_HEIGHT, grid(4.0));

    env.set(TEXTBOX_BORDER_RADIUS, 4.0);
    env.set(TEXTBOX_BORDER_WIDTH, 1.0);
    env.set(TEXTBOX_INSETS, Insets::uniform_xy(grid(1.2), grid(1.0)));

    env.set(SCROLLBAR_COLOR, env.get(GREY_300));
    env.set(SCROLLBAR_BORDER_COLOR, env.get(GREY_300));
    env.set(SCROLLBAR_MAX_OPACITY, 0.8);
    env.set(SCROLLBAR_FADE_DELAY, 1500u64);
    env.set(SCROLLBAR_WIDTH, 6.0);
    env.set(SCROLLBAR_PAD, 2.0);
    env.set(SCROLLBAR_RADIUS, 5.0);
    env.set(SCROLLBAR_EDGE_WIDTH, 1.0);

    env.set(WIDGET_PADDING_VERTICAL, grid(0.5));
    env.set(WIDGET_PADDING_HORIZONTAL, grid(1.0));
    env.set(WIDGET_CONTROL_COMPONENT_PADDING, grid(1.0));

    env.set(MENU_BUTTON_BG_ACTIVE, env.get(GREY_500));
    env.set(MENU_BUTTON_BG_INACTIVE, env.get(GREY_600));
    env.set(MENU_BUTTON_FG_ACTIVE, env.get(GREY_000));
    env.set(MENU_BUTTON_FG_INACTIVE, env.get(GREY_100));
}

fn setup_light_theme(env: &mut Env) {
    env.set(GREY_000, Color::grey8(0x00));
    env.set(GREY_100, Color::grey8(0x33));
    env.set(GREY_200, Color::grey8(0x4f));
    env.set(GREY_300, Color::grey8(0x82));
    env.set(GREY_400, Color::grey8(0xbd));
    env.set(GREY_500, Color::from_rgba32_u32(0xe5e6e7ff));
    env.set(GREY_600, Color::from_rgba32_u32(0xf5f6f7ff));
    env.set(GREY_700, Color::from_rgba32_u32(0xffffffff));
    env.set(BLUE_100, Color::rgb8(0x5c, 0xc4, 0xff));
    env.set(BLUE_200, Color::rgb8(0x00, 0x8d, 0xdd));

    env.set(RED, Color::rgba8(0xEB, 0x57, 0x57, 0xFF));

    env.set(LINK_HOT_COLOR, Color::rgba(0.0, 0.0, 0.0, 0.06));
    env.set(LINK_ACTIVE_COLOR, Color::rgba(0.0, 0.0, 0.0, 0.04));
    env.set(LINK_COLD_COLOR, Color::rgba(0.0, 0.0, 0.0, 0.0));
}

fn setup_dark_theme(env: &mut Env) {
    env.set(GREY_000, Color::grey8(0xff));
    env.set(GREY_100, Color::grey8(0xf2));
    env.set(GREY_200, Color::grey8(0xe0));
    env.set(GREY_300, Color::grey8(0xbd));
    env.set(GREY_400, Color::grey8(0x82));
    env.set(GREY_500, Color::grey8(0x4f));
    env.set(GREY_600, Color::grey8(0x33));
    env.set(GREY_700, Color::grey8(0x28));
    env.set(BLUE_100, Color::rgb8(0x00, 0x8d, 0xdd));
    env.set(BLUE_200, Color::rgb8(0x5c, 0xc4, 0xff));

    env.set(RED, Color::rgba8(0xEB, 0x57, 0x57, 0xFF));

    env.set(LINK_HOT_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.05));
    env.set(LINK_ACTIVE_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.025));
    env.set(LINK_COLD_COLOR, Color::rgba(1.0, 1.0, 1.0, 0.0));
}

fn setup_custom_theme(env: &mut Env, palette: &CustomTheme) {
    let parse = |hex: &str, default: Color| Color::from_hex_str(hex).unwrap_or(default);

    let background = parse(&palette.background, Color::grey8(0x18));
    let surface = parse(&palette.surface, Color::grey8(0x22));
    let primary_text = parse(&palette.primary_text, Color::grey8(0xee));
    let accent = parse(&palette.accent, Color::rgb8(0x1d, 0xb9, 0x54));
    let highlight = parse(&palette.highlight, Color::rgb8(0x3a, 0x7b, 0xd5));

    env.set(CUSTOM_COLOR_1, primary_text);
    env.set(CUSTOM_COLOR_2, surface);
    env.set(CUSTOM_COLOR_3, background);
    env.set(CUSTOM_COLOR_4, accent);
    env.set(CUSTOM_COLOR_5, highlight);

    env.set(GREY_000, primary_text);
    env.set(GREY_100, primary_text.with_alpha(0.9));
    env.set(GREY_200, primary_text.with_alpha(0.75));
    env.set(GREY_300, primary_text.with_alpha(0.55));
    env.set(GREY_400, primary_text.with_alpha(0.4));
    env.set(GREY_500, surface);
    env.set(GREY_600, background);
    env.set(GREY_700, background);

    env.set(BLUE_100, accent);
    env.set(BLUE_200, highlight);

    env.set(RED, Color::rgba8(0xEB, 0x57, 0x57, 0xFF));

    env.set(LINK_HOT_COLOR, accent.with_alpha(0.25));
    env.set(LINK_ACTIVE_COLOR, accent.with_alpha(0.15));
    env.set(LINK_COLD_COLOR, accent.with_alpha(0.0));
}

fn parse_font_family(family_name: &str) -> FontFamily {
    match family_name.to_lowercase().as_str() {
        "system ui" | "system-ui" => FontFamily::SYSTEM_UI,
        "serif" => FontFamily::SERIF,
        "sans-serif" => FontFamily::SANS_SERIF,
        "monospace" | "mono" => FontFamily::MONOSPACE,
        _ => FontFamily::new_unchecked(family_name),
    }
}
