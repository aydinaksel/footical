use leptos::prelude::*;
use leptos_router::components::{Redirect, Route, Router, Routes};
use leptos_router::path;

use crate::components::header::Header;
use crate::components::toast::ToastHost;
use crate::pages::admin::AdminPage;
use crate::pages::fines_admin::FinesAdminPage;
use crate::pages::fixtures::FixturesPage;
use crate::pages::home::Home;
use crate::pages::login::LoginPage;
use crate::pages::player::PlayerPage;
use crate::pages::squad_admin::SquadAdminPage;
use crate::pages::team::{TeamFinesPage, TeamFixturesPage};
use crate::pages::today::TodayPage;
use crate::tracked_team::provide_tracked_team;

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
    provide_tracked_team();

    view! {
        <Router>
            <ToastHost>
                <div class="min-h-screen bg-gray-50">
                    <Header />
                    <Routes fallback=|| "Page not found">
                        <Route path=path!("/") view=Home />
                        <Route path=path!("/fixtures") view=FixturesPage />
                        <Route path=path!("/today") view=TodayPage />
                        <Route
                            path=path!("/team")
                            view=|| view! { <Redirect path="/team/fixtures" /> }
                        />
                        <Route path=path!("/team/fixtures") view=TeamFixturesPage />
                        <Route path=path!("/team/fines") view=TeamFinesPage />
                        <Route path=path!("/team/player/:id") view=PlayerPage />
                        <Route path=path!("/admin/fines") view=FinesAdminPage />
                        <Route path=path!("/admin/squad") view=SquadAdminPage />
                        <Route path=path!("/admin/login") view=LoginPage />
                        <Route path=path!("/admin") view=AdminPage />
                    </Routes>
                </div>
            </ToastHost>
        </Router>
    }
    .into_any()
}
