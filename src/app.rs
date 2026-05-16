use freya::prelude::*;
use freya::radio::use_init_radio_station;

use crate::components::dashboard::Dashboard;
use crate::components::sidebar::Sidebar;
use crate::data::MessageData;
use crate::state::{AppChannel, AppState};

pub struct OcTokApp {
    pub messages: Vec<MessageData>,
}

impl App for OcTokApp {
    fn render(&self) -> impl IntoElement {
        use_init_theme(app_theme);

        let messages = self.messages.clone();
        use_init_radio_station::<AppState, AppChannel>(move || {
            let mut state = AppState {
                messages: messages.clone(),
                last_updated: Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()),
                ..AppState::default()
            };
            state.refresh_all();
            state
        });

        rect()
            .theme_background()
            .expanded()
            .horizontal()
            .child(Sidebar)
            .child(Dashboard)
    }
}

fn app_theme() -> Theme {
    let mut theme = dark_theme();
    theme.name = "oc-tok";
    theme.colors = ColorsSheet {
        primary: Color::from_rgb(255, 122, 0),
        secondary: Color::from_rgb(255, 160, 60),
        tertiary: Color::from_rgb(200, 90, 0),
        background: Color::from_rgb(20, 20, 21),
        surface_primary: Color::from_rgb(42, 42, 46),
        surface_secondary: Color::from_rgb(30, 30, 34),
        surface_tertiary: Color::from_rgb(24, 24, 27),
        text_primary: Color::from_rgb(240, 240, 240),
        text_secondary: Color::from_rgb(160, 160, 160),
        text_placeholder: Color::from_rgb(120, 120, 120),
        text_highlight: Color::from_rgb(255, 122, 0),
        border: Color::from_rgb(46, 46, 51),
        border_focus: Color::from_rgb(255, 122, 0),
        hover: Color::from_rgb(37, 37, 42),
        focus: Color::from_rgb(255, 122, 0),
        active: Color::from_rgb(42, 31, 21),
        info: Color::from_rgb(60, 130, 200),
        error: Color::from_rgb(200, 80, 80),
        success: Color::from_rgb(80, 200, 120),
        ..DARK_COLORS
    };
    theme
}
