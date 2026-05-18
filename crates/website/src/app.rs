use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::header::Header;
use crate::pages::admin::AdminPage;
use crate::pages::fixtures::FixturesPage;
use crate::pages::home::Home;
use crate::pages::login::LoginPage;
#[cfg(feature = "hydrate")]
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
                <link rel="stylesheet" href="/pkg/footical-website.css" />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let all_leagues: RwSignal<Vec<League>> = RwSignal::new(vec![]);
    let all_divisions: RwSignal<Vec<Division>> = RwSignal::new(vec![]);
    let all_teams: RwSignal<Vec<Team>> = RwSignal::new(vec![]);
    let all_fixtures: RwSignal<Vec<Fixture>> = RwSignal::new(vec![]);
    let tracked_team_id: RwSignal<Option<i32>> = RwSignal::new(read_tracked_team_id());
    let is_data_loaded: RwSignal<bool> = RwSignal::new(false);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(data) = get_leagues().await {
                all_leagues.set(data);
            }
            if let Ok(data) = get_divisions().await {
                all_divisions.set(data);
            }
            if let Ok(data) = get_teams().await {
                all_teams.set(data);
            }
            if let Ok(data) = get_fixtures().await {
                all_fixtures.set(data);
            }
            is_data_loaded.set(true);
        });
    }

    provide_context(all_leagues);
    provide_context(all_divisions);
    provide_context(all_teams);
    provide_context(all_fixtures);
    provide_context(tracked_team_id);
    provide_context(is_data_loaded);

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
    .into_any()
}
