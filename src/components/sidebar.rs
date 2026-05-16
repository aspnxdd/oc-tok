use freya::prelude::*;
use freya::radio::use_radio;

use crate::data::{DateRange, MessageData, RepoStats};
use crate::state::AppChannel;

#[derive(PartialEq)]
pub struct Sidebar;

impl Component for Sidebar {
    fn render(&self) -> impl IntoElement {
        let theme = use_theme();
        let radio = use_radio(AppChannel::All);
        let (is_loading, date_range, last_updated) = {
            let state = radio.read();
            (
                state.is_loading,
                state.date_range.clone(),
                state.last_updated.clone(),
            )
        };
        let colors = &theme.read().colors;
        let background = colors.surface_tertiary;
        let border = colors.border;

        rect()
            .width(Size::px(320.))
            .height(Size::fill())
            .background(background)
            .border(
                Border::new()
                    .width(BorderWidth {
                        right: 1.,
                        ..BorderWidth::default()
                    })
                    .fill(border),
            )
            .padding(Gaps::new_all(16.))
            .vertical()
            .spacing(16.)
            .content(Content::Flex)
            .child(section_heading("Repositories"))
            .child(RepoList)
            .child(DateRangeSection {
                current: date_range,
            })
            .child(RefreshSection {
                is_loading,
                last_updated,
            })
    }
}

#[derive(PartialEq)]
struct DateRangeSection {
    current: DateRange,
}

impl Component for DateRangeSection {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::All);

        let options = [
            ("7 Days", DateRange::Last7Days),
            ("30 Days", DateRange::Last30Days),
            ("All", DateRange::AllTime),
        ];

        rect()
            .vertical()
            .spacing(10.)
            .width(Size::fill())
            .child(section_heading("Date Range"))
            .child(
                options
                    .into_iter()
                    .fold(SegmentedButton::new(), |group, (label, range)| {
                        let selected = self.current == range;
                        let mut radio = radio;
                        group.child(
                            ButtonSegment::new()
                                .key(label)
                                .selected(selected)
                                .on_press(move |_| {
                                    let mut state = radio.write();
                                    if state.date_range == range {
                                        return;
                                    }
                                    state.date_range = range.clone();
                                    state.refresh_all();
                                })
                                .child(label),
                        )
                    }),
            )
    }
}

#[derive(PartialEq)]
struct RefreshSection {
    is_loading: bool,
    last_updated: Option<String>,
}

impl Component for RefreshSection {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::All);
        let placeholder_color = use_theme().read().colors.text_placeholder;

        let on_refresh = move |_| {
            let mut state = radio.write();
            state.is_loading = true;
            state.messages = MessageData::scan_opencode();
            state.refresh_all();
            state.last_updated = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
            state.is_loading = false;
        };

        rect()
            .vertical()
            .spacing(8.)
            .width(Size::fill())
            .child(
                Button::new()
                    .filled()
                    .expanded()
                    .enabled(!self.is_loading)
                    .on_press(on_refresh)
                    .child(if self.is_loading {
                        "Loading..."
                    } else {
                        "Update Data"
                    }),
            )
            .map(self.last_updated.as_deref(), |section, stamp| {
                section.child(
                    label()
                        .text(format!("Updated: {stamp}"))
                        .font_size(10.)
                        .color(placeholder_color),
                )
            })
    }
}

#[derive(PartialEq)]
struct RepoList;

impl Component for RepoList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::All);
        let (repo_stats, selected_repo) = {
            let state = radio.read();
            (state.repo_stats.clone(), state.selected_repo.clone())
        };
        let len = repo_stats.len();

        VirtualScrollView::new(move |index, _| {
            let stats = &repo_stats[index];
            RepoRow {
                selected: selected_repo.as_deref() == Some(stats.path.as_str()),
                stats: stats.clone(),
            }
            .into()
        })
        .length(len)
        .item_size(46.)
        .height(Size::flex(1.))
    }
}

#[derive(PartialEq)]
struct RepoRow {
    stats: RepoStats,
    selected: bool,
}

impl Component for RepoRow {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::All);
        let colors = &use_theme().read().colors;
        let path = self.stats.path.clone();

        let on_click = move |_| {
            let mut state = radio.write();
            state.selected_repo = if state.selected_repo.as_deref() == Some(path.as_str()) {
                None
            } else {
                Some(path.clone())
            };
            state.refresh_dashboard();
        };

        let (row_bg, stripe_color, name_color, count_color) = if self.selected {
            (
                colors.active,
                colors.primary,
                colors.text_primary,
                colors.secondary,
            )
        } else {
            (
                colors.surface_secondary,
                colors.surface_secondary,
                colors.text_secondary,
                colors.text_placeholder,
            )
        };

        rect()
            .key(self.stats.path.clone())
            .width(Size::fill())
            .height(Size::px(40.))
            .margin(Gaps::new(0., 0., 6., 0.))
            .background(row_bg)
            .corner_radius(CornerRadius::new_all(8.))
            .padding(Gaps::new(0., 12., 0., 12.))
            .on_press(on_click)
            .horizontal()
            .spacing(10.)
            .child(
                rect()
                    .width(Size::px(3.))
                    .height(Size::fill())
                    .background(stripe_color)
                    .corner_radius(CornerRadius::new_all(2.)),
            )
            .child(
                rect()
                    .vertical()
                    .center()
                    .expanded()
                    .child(
                        label()
                            .text(self.stats.name.clone())
                            .font_size(13.)
                            .color(name_color)
                            .text_overflow(TextOverflow::Ellipsis),
                    )
                    .child(
                        label()
                            .text(format!("{} msgs", self.stats.message_count))
                            .font_size(11.)
                            .color(count_color)
                            .text_overflow(TextOverflow::Ellipsis),
                    ),
            )
    }
}

fn section_heading(text: &str) -> impl IntoElement {
    label()
        .text(text.to_string())
        .font_size(13.)
        .font_weight(FontWeight::BOLD)
        .color(use_theme().read().colors.text_secondary)
}
