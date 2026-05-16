# OpenCode Token Dashboard — Agent Guide

## Overview

A native desktop dashboard for visualizing OpenCode token usage.
Built with [Freya](https://freyaui.dev/), a cross-platform Rust GUI library powered by Skia.

## Goal

Help users understand their AI coding assistant usage by repository, time range, model, and cost.

## Build & Test

```bash
cargo run   # Launch the app
cargo test  # Run unit tests
```

## Directory Structure

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point. Scans opencode data before launching the app window. |
| `src/app.rs` | Root component. Initializes the custom dark theme and the Freya Radio station. |
| `src/state.rs` | `AppState`, `AppChannel` enum, and state update helpers (`update_all_stats`, `update_dashboard_stats`). |
| `src/data.rs` | JSON parsing, aggregation logic, date filtering, and `RepoStats` computation. |
| `src/components/sidebar.rs` | Sidebar with repo list (`VirtualScrollView`), date range pills, and refresh button. |
| `src/components/dashboard.rs` | Main dashboard with `StatsCard` row and three plot components. |
| `src/components/stats_card.rs` | Reusable stat card component. |
| `src/components/plots.rs` | Three chart components: `DailyTokensPlot`, `ModelBreakdownPlot`, `DailyCostPlot`. |
| `skills/freya/SKILL.md` | Bundled Freya best-practices skill. Reference it when working on UI code. |

## Key Architectural Decisions

### Global State — Freya Radio

State lives in `AppState` and is broadcast via a single-variant `AppChannel::All`. Every write notifies every subscriber; consumers diff via Freya's PartialEq skip-rerender to avoid wasted work.

State mutations go through `AppState::refresh_all` (rebuilds `repo_stats` + `dashboard_stats`) or `AppState::refresh_dashboard` (rebuilds only `dashboard_stats`). Plots read precomputed `RepoStats::daily_sorted` / `models_sorted` so render never sorts.

### Data Source

On startup, `main.rs` calls `data::scan_opencode_data()`, which reads:

- `~/.local/share/opencode/storage/message/` — `msg_*.json` assistant messages
- `~/.local/share/opencode/storage/session/` — session metadata for repo path resolution

The refresh button in the sidebar re-runs this scan and rebuilds all stats.

### Plot Rendering

Charts are rendered with **Plotters** via Freya's `PlotSkiaBackend` inside a `canvas(RenderCallback::new(...))` element. Each plot component clones its data before moving it into the render callback closure.

### Theming

A custom dark theme with orange accents is defined in `app.rs` (`app_theme()`). It overrides `dark_theme()` colors. Built-in theme helpers (e.g., `.theme_background()`) are used where possible; hardcoded colors are used for accent and card surfaces.

## Conventions for This Repo

- Keep new components in `src/components/` and re-export them in `src/components/mod.rs`.
- Clone data before passing it into `RenderCallback` closures for Plotters canvases.
- Use `.key()` on dynamic list items (see `RepoList` inside `VirtualScrollView`).
- Prefer the existing `StatsCard` and `Sidebar` component patterns for consistency.
- When adding new stats, extend `RepoStats` in `data.rs` and compute the values in `aggregate_repos()`.
- For Freya-specific patterns (hooks, state management, theming, testing), consult `skills/freya/SKILL.md`.

## How to Extend

| Task | Where to Change |
|------|----------------|
| Add a new filter (e.g., by model) | Extend `AppState` and `aggregate_repos()`. Add UI controls in `Sidebar`. |
| Add a new plot type | Create a new component in `src/components/plots.rs` following the `canvas(RenderCallback::new(...))` + `PlotSkiaBackend` pattern. Register it in `Dashboard`. |
| Add a new stat to cards | Extend `RepoStats`, compute in `aggregate_repos()`, display in `Dashboard` via `StatsCard`. |
| Add a new data field | Extend `MessageJson` and `MessageData`, handle defaults in parsing. |

## External Reference

- Freya docs: https://docs.rs/freya/
- Freya llms.txt: https://freyaui.dev/llms.txt
