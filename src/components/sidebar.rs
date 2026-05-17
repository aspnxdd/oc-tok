use freya::prelude::*;
use freya::radio::use_radio;
use freya::router::{RouterContext, use_route};

use crate::data::{DateRange, MessageData, RepoStats};
use crate::route::Route;
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
        let repo_stats = radio.read().repo_stats.clone();
        let len = repo_stats.len();

        VirtualScrollView::new(move |index, _| {
            RepoRow {
                stats: repo_stats[index].clone(),
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
}

impl Component for RepoRow {
    fn render(&self) -> impl IntoElement {
        let current_route = use_route::<Route>();
        let target = Route::for_repo_path(&self.stats.path);
        let is_active = current_route == target;
        let count_color = use_theme().read().colors.text_secondary;

        let on_press = move |_| {
            let next = if is_active { Route::Home } else { target.clone() };
            let _ = RouterContext::get().push(next);
        };

        ActivableRoute::new(
            Route::for_repo_path(&self.stats.path),
            SideBarItem::new()
                .key(self.stats.path.clone())
                .on_press(on_press)
                .child(
                    rect()
                        .vertical()
                        .child(
                            label()
                                .text(self.stats.name.clone())
                                .font_size(13.)
                                .text_overflow(TextOverflow::Ellipsis),
                        )
                        .child(
                            label()
                                .text(format!("{} msgs", self.stats.message_count))
                                .font_size(11.)
                                .color(count_color)
                                .text_overflow(TextOverflow::Ellipsis),
                        ),
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
