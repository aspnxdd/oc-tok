use freya::prelude::*;
use freya::radio::*;

use crate::data::DateRange;
use crate::state::{AppChannel, update_all_stats, update_dashboard_stats};

#[derive(PartialEq)]
pub struct Sidebar;

impl Component for Sidebar {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Selection);
        let state = radio.read();
        let is_loading = state.is_loading;
        let date_range = state.date_range.clone();
        let last_updated = state.last_updated.clone();
        drop(state);

        let mut radio_7 = radio.clone();
        let on_7days = move |_| {
            let mut s = radio_7.write();
            s.date_range = DateRange::Last7Days;
            update_all_stats(&mut s);
        };

        let mut radio_30 = radio.clone();
        let on_30days = move |_| {
            let mut s = radio_30.write();
            s.date_range = DateRange::Last30Days;
            update_all_stats(&mut s);
        };

        let mut radio_all = radio.clone();
        let on_all = move |_| {
            let mut s = radio_all.write();
            s.date_range = DateRange::AllTime;
            update_all_stats(&mut s);
        };

        let mut radio_update = radio.clone();
        let on_update = move |_| {
            let mut s = radio_update.write();
            s.is_loading = true;
            let messages = crate::data::scan_opencode_data();
            s.messages = messages;
            update_all_stats(&mut s);
            s.is_loading = false;
            s.last_updated = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
        };

        rect()
            .width(Size::px(320.))
            .height(Size::fill())
            .background((24, 24, 27))
            .border(
                Border::new()
                    .width(BorderWidth {
                        top: 0.,
                        right: 1.,
                        bottom: 0.,
                        left: 0.,
                    })
                    .fill((46, 46, 51)),
            )
            .padding(Gaps::new(16., 16., 16., 16.))
            .vertical()
            .spacing(16.)
            .child(
                label()
                    .text("Repositories")
                    .font_size(13.)
                    .font_weight(FontWeight::BOLD)
                    .color((160, 160, 160)),
            )
            .child(rect().expanded().vertical().child(RepoList {}))
            .child(
                rect()
                    .vertical()
                    .spacing(10.)
                    .width(Size::fill())
                    .child(
                        label()
                            .text("Date Range")
                            .font_size(13.)
                            .font_weight(FontWeight::BOLD)
                            .color((160, 160, 160)),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .spacing(6.)
                            .width(Size::fill())
                            .child(
                                rect()
                                    .background(if date_range == DateRange::Last7Days {
                                        (255, 122, 0)
                                    } else {
                                        (42, 42, 46)
                                    })
                                    .corner_radius(CornerRadius::new_all(99.))
                                    .center()
                                    .padding(Gaps::new(8., 14., 8., 14.))
                                    .on_mouse_up(on_7days)
                                    .child(
                                        label()
                                            .text("7 Days")
                                            .color(if date_range == DateRange::Last7Days {
                                                (255, 255, 255)
                                            } else {
                                                (160, 160, 160)
                                            })
                                            .font_size(12.)
                                            .font_weight(FontWeight::SEMI_BOLD),
                                    ),
                            )
                            .child(
                                rect()
                                    .background(if date_range == DateRange::Last30Days {
                                        (255, 122, 0)
                                    } else {
                                        (42, 42, 46)
                                    })
                                    .corner_radius(CornerRadius::new_all(99.))
                                    .center()
                                    .padding(Gaps::new(8., 14., 8., 14.))
                                    .on_mouse_up(on_30days)
                                    .child(
                                        label()
                                            .text("30 Days")
                                            .color(if date_range == DateRange::Last30Days {
                                                (255, 255, 255)
                                            } else {
                                                (160, 160, 160)
                                            })
                                            .font_size(12.)
                                            .font_weight(FontWeight::SEMI_BOLD),
                                    ),
                            )
                            .child(
                                rect()
                                    .background(if date_range == DateRange::AllTime {
                                        (255, 122, 0)
                                    } else {
                                        (42, 42, 46)
                                    })
                                    .corner_radius(CornerRadius::new_all(99.))
                                    .center()
                                    .padding(Gaps::new(8., 14., 8., 14.))
                                    .on_mouse_up(on_all)
                                    .child(
                                        label()
                                            .text("All")
                                            .color(if date_range == DateRange::AllTime {
                                                (255, 255, 255)
                                            } else {
                                                (160, 160, 160)
                                            })
                                            .font_size(12.)
                                            .font_weight(FontWeight::SEMI_BOLD),
                                    ),
                            ),
                    ),
            )
            .child(
                rect()
                    .vertical()
                    .spacing(8.)
                    .width(Size::fill())
                    .child(
                        rect()
                            .background(if is_loading {
                                (120, 70, 0)
                            } else {
                                (255, 122, 0)
                            })
                            .corner_radius(CornerRadius::new_all(10.))
                            .center()
                            .padding(Gaps::new(10., 16., 10., 16.))
                            .width(Size::fill())
                            .on_mouse_up(on_update)
                            .child(
                                label()
                                    .text(if is_loading {
                                        "Loading..."
                                    } else {
                                        "Update Data"
                                    })
                                    .color((255, 255, 255))
                                    .font_size(13.)
                                    .font_weight(FontWeight::BOLD),
                            ),
                    )
                    .maybe(last_updated.is_some(), |r| {
                        r.child(
                            label()
                                .text(format!("Updated: {}", last_updated.as_ref().unwrap()))
                                .font_size(10.)
                                .color((120, 120, 120)),
                        )
                    }),
            )
    }
}

#[derive(PartialEq)]
struct RepoList;

impl Component for RepoList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Selection);
        let state = radio.read();
        let repo_stats = state.repo_stats.clone();
        let selected_repo = state.selected_repo.clone();
        let repo_stats_len = repo_stats.len();
        drop(state);

        VirtualScrollView::new(move |i, _| {
                if i >= repo_stats.len() {
                    return rect().into();
                }
                let repo = &repo_stats[i];
                let path = repo.path.clone();
                let name = repo.name.clone();
                let is_selected = selected_repo.as_ref() == Some(&path);
                let count = repo.message_count;

                let mut radio_click = radio.clone();
                let path_for_click = path.clone();
                let on_click = move |_| {
                    let mut s = radio_click.write();
                    if s.selected_repo.as_ref() == Some(&path_for_click) {
                        s.selected_repo = None;
                    } else {
                        s.selected_repo = Some(path_for_click.clone());
                    }
                    update_dashboard_stats(&mut s);
                };

                rect()
                    .key(path.clone())
                    .width(Size::fill())
                    .height(Size::px(40.))
                    .background(if is_selected {
                        (42, 31, 21)
                    } else {
                        (30, 30, 34)
                    })
                    .corner_radius(CornerRadius::new_all(8.))
                    .padding(Gaps::new(0., 12., 0., 12.))
                    .on_mouse_up(on_click)
                    .child(
                        rect()
                            .horizontal()
                            .spacing(10.)
                            .width(Size::fill())
                            .child(
                                rect()
                                    .width(Size::px(3.))
                                    .height(Size::fill())
                                    .background(if is_selected {
                                        (255, 122, 0)
                                    } else {
                                        (30, 30, 34)
                                    })
                                    .corner_radius(CornerRadius::new_all(2.)),
                            )
                            .child(
                                rect()
                                    .vertical()
                                    .center()
                                    .expanded()
                                    .child(
                                        label()
                                            .text(format!("{}", name))
                                            .font_size(13.)
                                            .color(if is_selected {
                                                (255, 255, 255)
                                            } else {
                                                (180, 180, 180)
                                            })
                                            .text_overflow(TextOverflow::Ellipsis),
                                    )
                                    .child(
                                        label()
                                            .text(format!("{} msgs", count))
                                            .font_size(11.)
                                            .color(if is_selected {
                                                (255, 160, 60)
                                            } else {
                                                (120, 120, 120)
                                            })
                                            .text_overflow(TextOverflow::Ellipsis),
                                    ),
                            ),
                    )
                    .into()
            })
            .length(repo_stats_len)
            .item_size(40.)
            .height(Size::fill())
    }
}
