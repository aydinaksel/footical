mod components;
mod pages;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use components::header::Header;
use pages::calendar::CalendarPage;
use pages::home::Home;

fn main() {
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
                </Routes>
            </div>
        </Router>
    }
}
