use freya::prelude::*;
use freya::radio::*;

use crate::components::plots::{DailyCostPlot, DailyTokensPlot, ModelBreakdownPlot};
use crate::components::stats_card::StatsCard;
use crate::state::AppChannel;

#[derive(PartialEq)]
pub struct Dashboard;

impl Component for Dashboard {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Data);
        let state = radio.read();
        let stats = state.dashboard_stats.clone();
        let has_data = stats.message_count > 0;
        drop(state);

        rect()
            .expanded()
            .background((20, 20, 21))
            .child(
                ScrollView::new()
                    .expanded()
                    .child(
                        rect()
                            .padding(Gaps::new(24., 24., 24., 24.))
                            .vertical()
                            .spacing(20.)
                            .child(
                                rect()
                                    .vertical()
                                    .spacing(4.)
                                    .child(
                                        label()
                                            .text("Dashboard")
                                            .font_size(22.)
                                            .font_weight(FontWeight::BOLD)
                                            .color((240, 240, 240)),
                                    )
                                    .child(
                                        rect()
                                            .width(Size::px(40.))
                                            .height(Size::px(3.))
                                            .background((255, 122, 0))
                                            .corner_radius(CornerRadius::new_all(2.)),
                                    ),
                            )
                            .maybe(has_data, |r| {
                                r.child(
                                    rect()
                                        .horizontal()
                                        .spacing(14.)
                                        .width(Size::fill())
                                        .child(StatsCard {
                                            label: "Messages".to_string(),
                                            value: format_num(stats.message_count as f64),
                                            color: Color::from_rgb(200, 200, 200),
                                        })
                                        .child(StatsCard {
                                            label: "Input Tokens".to_string(),
                                            value: format_num(stats.total_input as f64),
                                            color: Color::from_rgb(60, 130, 200),
                                        })
                                        .child(StatsCard {
                                            label: "Output Tokens".to_string(),
                                            value: format_num(stats.total_output as f64),
                                            color: Color::from_rgb(200, 80, 80),
                                        })
                                        .child(StatsCard {
                                            label: "Total Cost".to_string(),
                                            value: format!("{:.4}", stats.total_cost),
                                            color: Color::from_rgb(80, 200, 120),
                                        }),
                                )
                                .child(
                                    rect()
                                        .vertical()
                                        .spacing(14.)
                                        .width(Size::fill())
                                        .child(
                                            rect()
                                                .background((30, 30, 34))
                                                .corner_radius(CornerRadius::new_all(12.))
                                                .padding(Gaps::new(12., 12., 12., 12.))
                                                .height(Size::px(220.))
                                                .width(Size::fill())
                                                .child(DailyTokensPlot {
                                                    daily: stats.daily.clone(),
                                                }),
                                        )
                                        .child(
                                            rect()
                                                .background((30, 30, 34))
                                                .corner_radius(CornerRadius::new_all(12.))
                                                .padding(Gaps::new(12., 12., 12., 12.))
                                                .height(Size::px(220.))
                                                .width(Size::fill())
                                                .child(ModelBreakdownPlot {
                                                    model_breakdown: stats.model_breakdown.clone(),
                                                }),
                                        )
                                        .child(
                                            rect()
                                                .background((30, 30, 34))
                                                .corner_radius(CornerRadius::new_all(12.))
                                                .padding(Gaps::new(12., 12., 12., 12.))
                                                .height(Size::px(220.))
                                                .width(Size::fill())
                                                .child(DailyCostPlot {
                                                    daily: stats.daily.clone(),
                                                }),
                                        ),
                                )
                            })
                            .maybe(!has_data, |r| {
                                r.child(
                                    rect()
                                        .height(Size::fill())
                                        .center()
                                        .child(
                                            label()
                                                .text("No data for the selected filters.")
                                                .color((150, 150, 150))
                                                .font_size(14.),
                                        ),
                                )
                            }),
                    ),
            )
    }
}

fn format_num(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}K", n / 1_000.0)
    } else {
        format!("{:.0}", n)
    }
}
