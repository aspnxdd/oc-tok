use freya::prelude::*;
use freya::radio::*;

use crate::components::dashboard::Dashboard;
use crate::components::sidebar::Sidebar;
use crate::state::{AppChannel, AppState, update_all_stats};
use crate::data::MessageData;

fn app_theme() -> Theme {
    let mut theme = dark_theme();
    theme.colors.primary = Color::from_rgb(255, 122, 0);
    theme.colors.secondary = Color::from_rgb(255, 160, 60);
    theme.colors.tertiary = Color::from_rgb(200, 90, 0);
    theme.colors.background = Color::from_rgb(20, 20, 21);
    theme.colors.surface_primary = Color::from_rgb(42, 42, 46);
    theme.colors.surface_secondary = Color::from_rgb(30, 30, 34);
    theme.colors.surface_tertiary = Color::from_rgb(24, 24, 27);
    theme.colors.text_primary = Color::from_rgb(240, 240, 240);
    theme.colors.text_secondary = Color::from_rgb(160, 160, 160);
    theme.colors.text_placeholder = Color::from_rgb(120, 120, 120);
    theme.colors.text_highlight = Color::from_rgb(255, 122, 0);
    theme.colors.border = Color::from_rgb(46, 46, 51);
    theme.colors.border_focus = Color::from_rgb(255, 122, 0);
    theme.colors.hover = Color::from_rgb(37, 37, 42);
    theme.colors.focus = Color::from_rgb(255, 122, 0);
    theme.colors.active = Color::from_rgb(42, 31, 21);
    theme
}

pub fn app() -> impl IntoElement {
    let _theme = use_init_theme(app_theme);

    let messages: Vec<MessageData> = crate::INITIAL_MESSAGES
        .get()
        .cloned()
        .unwrap_or_default();

    use_init_radio_station::<AppState, AppChannel>(move || {
        let mut state = AppState::default();
        state.messages = messages;
        update_all_stats(&mut state);
        state.last_updated = Some(
            chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        );
        state
    });

    rect()
        .theme_background()
        .expanded()
        .horizontal()
        .child(Sidebar {})
        .child(Dashboard {})
}
