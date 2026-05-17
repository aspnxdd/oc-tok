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
| `src/app.rs` | `OcTokApp` root: initializes theme + Radio station and renders the `Router<Route>`. |
| `src/route.rs` | `Route` enum (Home, Repo) + `AppLayout` (sidebar + Outlet) + per-route components that compute their slice via `RepoStats::for_dashboard`. |
| `src/state.rs` | `AppState` (messages, date_range, is_loading, last_updated, repo_stats) + single `AppChannel::All`. |
| `src/data.rs` | JSON parsing, aggregation, date filtering, `RepoStats` computation (with precomputed sorted vecs). |
| `src/components/sidebar.rs` | Sidebar with repo list (`VirtualScrollView` of `SideBarItem` rows wrapped in `ActivableRoute<Route>`), date range `SegmentedButton`, refresh `Button`. |
| `src/components/dashboard.rs` | `Dashboard { stats: RepoStats }` — takes its data as a prop and renders the cards + plots. |
| `src/components/stats_card.rs` | Reusable stat card. |
| `src/components/plots.rs` | Three chart components: `DailyTokensPlot`, `ModelBreakdownPlot`, `DailyCostPlot`. |
| `skills/freya/SKILL.md` | Bundled Freya best-practices skill. Reference it when working on UI code. |

## Key Architectural Decisions

### Global State — Freya Radio

State lives in `AppState` and is broadcast via a single-variant `AppChannel::All`. Every write notifies every subscriber; consumers diff via Freya's PartialEq skip-rerender to avoid wasted work.

State mutations go through `AppState::refresh_all`, which rebuilds `repo_stats` from raw messages (after a date-range change or refresh). Selection is **not** in state — it's a URL route (`Route::Home` vs `Route::Repo { path }`). The per-route components in `src/route.rs` call `RepoStats::for_dashboard` to derive their slice. Plots read precomputed `RepoStats::daily_sorted` / `models_sorted` so render never sorts.

### Routing — freya-router

Routes are defined in `src/route.rs` with `#[derive(Routable)]`. `AppLayout` is the shared chrome (sidebar + `Outlet<Route>`); per-route components render inside the outlet. Sidebar rows wrap a `SideBarItem` in `ActivableRoute<Route>` so the built-in active styling lights up automatically when the current route matches. Navigation: `RouterContext::get().push(Route::...)`. Filesystem repo paths map to URL catch-all segments via `Route::for_repo_path`.

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
- When adding new stats, extend `RepoStats` in `data.rs` and compute the values in `RepoStats::aggregate`.
- For Freya-specific patterns (hooks, state management, theming, testing), consult `skills/freya/SKILL.md`.

## How to Extend

| Task | Where to Change |
|------|----------------|
| Add a new filter (e.g., by model) | Extend `AppState` and `RepoStats::aggregate`. Add UI controls in `Sidebar`. |
| Add a new plot type | Create a new component in `src/components/plots.rs` following the `plot_canvas(palette.fill, draw)` pattern. Register it in `Dashboard`. |
| Add a new stat to cards | Extend `RepoStats`, compute in `RepoStats::aggregate`, display in `Dashboard` via `StatsCard`. |
| Add a new data field | Extend `MessageJson` and `MessageData`, handle defaults in `MessageJson::into_assistant_data`. |
| Add a new route | Add a variant to `Route` in `src/route.rs` with a `#[route("...")]` attribute and a matching component. |

## External Reference

- Freya docs: https://docs.rs/freya/
- Freya llms.txt: https://freyaui.dev/llms.txt
