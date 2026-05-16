mod app;
mod components;
mod data;
mod route;
mod state;

use freya::prelude::*;

use crate::app::OcTokApp;
use crate::data::MessageData;

fn main() {
    let messages = MessageData::scan_opencode();
    launch(
        LaunchConfig::new()
            .with_window(WindowConfig::new_app(OcTokApp { messages }).with_size(1400., 900.)),
    );
}
