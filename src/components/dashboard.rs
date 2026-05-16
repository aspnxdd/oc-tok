use freya::prelude::*;
use freya::radio::use_radio;

use crate::components::plots::{DailyCostPlot, DailyTokensPlot, ModelBreakdownPlot};
use crate::components::stats_card::StatsCard;
use crate::data::RepoStats;
use crate::state::AppChannel;

#[derive(PartialEq)]
pub struct Dashboard;

impl Component for Dashboard {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::All);
        let stats = radio.read().dashboard_stats.clone();
        let has_data = stats.message_count > 0;

        rect().expanded().theme_background().child(
            ScrollView::new().expanded().child(
                rect()
                    .padding(Gaps::new_all(24.))
                    .vertical()
                    .spacing(20.)
                    .child(DashboardHeader)
                    .maybe(has_data, |column| {
                        column
                            .child(StatsRow {
                                stats: stats.clone(),
                            })
                            .child(PlotsColumn {
                                stats: stats.clone(),
                            })
                    })
                    .maybe(!has_data, |column| column.child(EmptyState)),
            ),
        )
    }
}

#[derive(PartialEq)]
struct DashboardHeader;

impl Component for DashboardHeader {
    fn render(&self) -> impl IntoElement {
        let primary = use_theme().read().colors.primary;
        rect()
            .vertical()
            .spacing(4.)
            .child(
                label()
                    .text("Dashboard")
                    .font_size(22.)
                    .font_weight(FontWeight::BOLD)
                    .theme_color(),
            )
            .child(
                rect()
                    .width(Size::px(40.))
                    .height(Size::px(3.))
                    .background(primary)
                    .corner_radius(CornerRadius::new_all(2.)),
            )
    }
}

#[derive(PartialEq)]
struct StatsRow {
    stats: RepoStats,
}

impl Component for StatsRow {
    fn render(&self) -> impl IntoElement {
        let colors = &use_theme().read().colors;
        let cards = [
            (
                "Messages",
                format_num(self.stats.message_count as f64),
                colors.text_primary,
            ),
            (
                "Input Tokens",
                format_num(self.stats.total_input as f64),
                colors.info,
            ),
            (
                "Output Tokens",
                format_num(self.stats.total_output as f64),
                colors.error,
            ),
            (
                "Total Cost",
                format!("{:.4}", self.stats.total_cost),
                colors.success,
            ),
        ];

        cards.into_iter().fold(
            rect()
                .horizontal()
                .spacing(14.)
                .width(Size::fill())
                .content(Content::Flex),
            |row, (label, value, color)| {
                row.child(StatsCard {
                    label: label.to_string(),
                    value,
                    color,
                })
            },
        )
    }
}

#[derive(PartialEq)]
struct PlotsColumn {
    stats: RepoStats,
}

impl Component for PlotsColumn {
    fn render(&self) -> impl IntoElement {
        rect()
            .vertical()
            .spacing(14.)
            .width(Size::fill())
            .child(plot_card(DailyTokensPlot {
                daily: self.stats.daily_sorted.clone(),
            }))
            .child(plot_card(ModelBreakdownPlot {
                models: self.stats.models_sorted.clone(),
            }))
            .child(plot_card(DailyCostPlot {
                daily: self.stats.daily_sorted.clone(),
            }))
    }
}

fn plot_card(plot: impl Component + 'static) -> impl IntoElement {
    Card::new()
        .height(Size::px(220.))
        .width(Size::fill())
        .child(plot)
}

#[derive(PartialEq)]
struct EmptyState;

impl Component for EmptyState {
    fn render(&self) -> impl IntoElement {
        rect().height(Size::fill()).center().child(
            label()
                .text("No data for the selected filters.")
                .color(use_theme().read().colors.text_secondary)
                .font_size(14.),
        )
    }
}

fn format_num(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}K", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}
