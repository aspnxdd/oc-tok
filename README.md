# OpenCode Token Dashboard

A native desktop dashboard for visualizing OpenCode token usage, built with [Freya](https://freyaui.dev/) (Rust + Skia).

## Features

- **Repository-scoped tracking** — browse usage per project via the sidebar
- **Date range filtering** — 7 days, 30 days, or all time
- **Real-time stat cards** — messages, input/output tokens, and total cost
- **Interactive visualizations**
  - Daily Tokens (grouped bar chart)
  - Model Breakdown (horizontal bar chart)
  - Daily Cost (line chart)
- **One-click refresh** — reload data from local OpenCode storage

## Requirements

- Rust toolchain (latest stable)
- Existing OpenCode data at `~/.local/share/opencode/storage/`

## Quick Start

```bash
cargo run
```

The app launches a 1400x900 window and reads your local message/session JSON files on startup.

## Data Source

The dashboard scans:

- `~/.local/share/opencode/storage/message/` — individual message JSONs (`msg_*.json`)
- `~/.local/share/opencode/storage/session/` — session metadata mapping session IDs to working directories

Only assistant messages are counted. Token fields (`input`, `output`, `reasoning`, `cache`) and `cost` are aggregated per repository and date.

## Tech Stack

- **Rust** — language
- **Freya** — native UI framework (reactive, declarative, Skia-rendered)
- **Plotters** — charting library
- **Skia** — 2D graphics engine (via Freya)

## License

MIT
