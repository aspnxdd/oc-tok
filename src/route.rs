use freya::prelude::*;
use freya::radio::use_radio;
use freya::router::*;

use crate::components::dashboard::Dashboard;
use crate::components::sidebar::Sidebar;
use crate::data::RepoStats;
use crate::state::AppChannel;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Home,
        #[route("/repo/:..path")]
        Repo { path: Vec<String> },
}

impl Route {
    pub fn for_repo_path(repo_path: &str) -> Self {
        Self::Repo {
            path: repo_path
                .trim_start_matches('/')
                .split('/')
                .filter(|seg| !seg.is_empty())
                .map(String::from)
                .collect(),
        }
    }
}

#[derive(PartialEq)]
pub struct AppLayout;

impl Component for AppLayout {
    fn render(&self) -> impl IntoElement {
        rect()
            .theme_background()
            .expanded()
            .horizontal()
            .child(Sidebar)
            .child(Outlet::<Route>::new())
    }
}

#[derive(PartialEq)]
pub struct Home;

impl Component for Home {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::All);
        let stats = RepoStats::for_dashboard(&radio.read().repo_stats, None);
        Dashboard { stats }
    }
}

#[derive(PartialEq)]
pub struct Repo {
    pub path: Vec<String>,
}

impl Component for Repo {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::All);
        let full_path = format!("/{}", self.path.join("/"));
        let stats = RepoStats::for_dashboard(&radio.read().repo_stats, Some(&full_path));
        Dashboard { stats }
    }
}
