mod clipboard;
mod components;
mod pages;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use components::header::Header;
use pages::calendar::CalendarPage;
use pages::fixtures::FixturesPage;
use pages::home::Home;
use pages::subscribe::SubscribePage;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
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
