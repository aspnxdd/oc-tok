mod app;
mod components;
mod data;
mod state;

use freya::prelude::*;
use std::sync::OnceLock;

use crate::data::MessageData;

static INITIAL_MESSAGES: OnceLock<Vec<MessageData>> = OnceLock::new();

fn main() {
    let messages = data::scan_opencode_data();
    let _ = INITIAL_MESSAGES.set(messages);
    launch(LaunchConfig::new().with_window(
        WindowConfig::new(app::app).with_size(1400., 900.),
    ));
}
