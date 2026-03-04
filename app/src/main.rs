mod clipboard;
mod components;
mod pages;
mod types;

use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use wasm_bindgen_futures::spawn_local;

use components::header::Header;
use pages::calendar::CalendarPage;
use pages::fixtures::FixturesPage;
use pages::home::Home;
use pages::subscribe::SubscribePage;
use types::{Division, League, Team, read_tracked_team_id};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let all_leagues: RwSignal<Vec<League>> = RwSignal::new(vec![]);
    let all_divisions: RwSignal<Vec<Division>> = RwSignal::new(vec![]);
    let all_teams: RwSignal<Vec<Team>> = RwSignal::new(vec![]);
    let tracked_team_id: RwSignal<Option<i32>> = RwSignal::new(read_tracked_team_id());
    let is_data_loading: RwSignal<bool> = RwSignal::new(true);

    provide_context(all_leagues);
    provide_context(all_divisions);
    provide_context(all_teams);
    provide_context(tracked_team_id);
    provide_context(is_data_loading);

    spawn_local(async move {
        let leagues = Request::get("https://data.footical.club/leagues.json")
            .send()
            .await
            .unwrap()
            .json::<Vec<League>>()
            .await
            .unwrap_or_default();
        all_leagues.set(leagues);

        let divisions = Request::get("https://data.footical.club/divisions.json")
            .send()
            .await
            .unwrap()
            .json::<Vec<Division>>()
            .await
            .unwrap_or_default();
        all_divisions.set(divisions);

        let teams = Request::get("https://data.footical.club/teams.json")
            .send()
            .await
            .unwrap()
            .json::<Vec<Team>>()
            .await
            .unwrap_or_default();
        all_teams.set(teams);

        is_data_loading.set(false);
    });

    view! {
        <Router>
            <div class="min-h-screen bg-gray-50">
                <Header />
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/calendar") view=CalendarPage />
                    <Route path=path!("/fixtures") view=FixturesPage />
                    <Route path=path!("/subscribe") view=SubscribePage />
                </Routes>
            </div>
        </Router>
    }
}
