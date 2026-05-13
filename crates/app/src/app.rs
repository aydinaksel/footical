use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::header::Header;
use crate::pages::admin::AdminPage;
use crate::pages::fixtures::FixturesPage;
use crate::pages::home::Home;
use crate::pages::login::LoginPage;
use crate::server::data::{get_divisions, get_fixtures, get_leagues, get_teams};
use crate::types::{Division, Fixture, League, Team, read_tracked_team_id};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Footical"</title>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options />
                <link rel="stylesheet" href="/pkg/footical-app.css" />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let leagues_resource = Resource::new(|| (), |_| get_leagues());
    let divisions_resource = Resource::new(|| (), |_| get_divisions());
    let teams_resource = Resource::new(|| (), |_| get_teams());
    let fixtures_resource = Resource::new(|| (), |_| get_fixtures());

    let all_leagues: RwSignal<Vec<League>> = RwSignal::new(vec![]);
    let all_divisions: RwSignal<Vec<Division>> = RwSignal::new(vec![]);
    let all_teams: RwSignal<Vec<Team>> = RwSignal::new(vec![]);
    let all_fixtures: RwSignal<Vec<Fixture>> = RwSignal::new(vec![]);
    let tracked_team_id: RwSignal<Option<i32>> = RwSignal::new(read_tracked_team_id());
    let is_data_loading: RwSignal<bool> = RwSignal::new(true);

    Effect::new(move || {
        if let Some(Ok(leagues)) = leagues_resource.get() {
            all_leagues.set(leagues);
        }
        if let Some(Ok(divisions)) = divisions_resource.get() {
            all_divisions.set(divisions);
        }
        if let Some(Ok(teams)) = teams_resource.get() {
            all_teams.set(teams);
        }
        if let Some(Ok(fixtures)) = fixtures_resource.get() {
            all_fixtures.set(fixtures);
        }
        if leagues_resource.get().is_some()
            && divisions_resource.get().is_some()
            && teams_resource.get().is_some()
            && fixtures_resource.get().is_some()
        {
            is_data_loading.set(false);
        }
    });

    provide_context(all_leagues);
    provide_context(all_divisions);
    provide_context(all_teams);
    provide_context(all_fixtures);
    provide_context(tracked_team_id);
    provide_context(is_data_loading);

    view! {
        <Router>
            <div class="min-h-screen bg-gray-50">
                <Header />
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/fixtures") view=FixturesPage />
                    <Route path=path!("/admin/login") view=LoginPage />
                    <Route path=path!("/admin") view=AdminPage />
                </Routes>
            </div>
        </Router>
    }
}
