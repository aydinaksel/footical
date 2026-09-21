#![expect(
    clippy::mem_forget,
    reason = "the island macro forgets the render state it built on the server"
)]

use crate::components::admin_only::AdminOnly;
use crate::components::browser_navigation::navigate_to;
use crate::server::auth::Logout;
use crate::server::scraper::{get_scrape_status, ScrapeStatus, TriggerScrape};
use leptos::prelude::*;

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <AdminOnly>
            <ScraperDashboard />
        </AdminOnly>
    }
    .into_any()
}

#[island]
fn ScraperDashboard() -> impl IntoView {
    let trigger_scrape = ServerAction::<TriggerScrape>::new();
    let logout = ServerAction::<Logout>::new();
    let scrape_status = Resource::new(
        move || trigger_scrape.version().get(),
        |_| get_scrape_status(),
    );

    Effect::new(move |_| {
        if matches!(logout.value().get(), Some(Ok(()))) {
            navigate_to("/admin/login");
        }
    });

    let is_scrape_running = move || {
        scrape_status
            .get()
            .and_then(|status| status.ok())
            .map(|status| status.is_running)
            .unwrap_or(false)
    };

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-lg">
                <div class="bg-white rounded-xl shadow-md overflow-hidden">
                    <div class="px-6 py-5 border-b border-gray-100 flex items-center justify-between">
                        <div>
                            <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                                "Admin"
                            </p>
                            <h1 class="text-xl font-bold text-gray-800 mt-0.5">
                                "Scraper Dashboard"
                            </h1>
                        </div>
                        <div class="flex items-center gap-4">
                            <a
                                href="/admin/squad"
                                class="text-sm text-blue-600 hover:text-blue-700"
                            >
                                "Squad"
                            </a>
                            <a
                                href="/admin/fines"
                                class="text-sm text-blue-600 hover:text-blue-700"
                            >
                                "Fines"
                            </a>
                            <button
                                class="text-sm text-gray-400 hover:text-gray-600 transition-colors cursor-pointer"
                                on:click=move |_| {
                                    logout.dispatch(Logout {});
                                }
                            >
                                "Logout"
                            </button>
                        </div>
                    </div>

                    <div class="px-6 py-5 border-b border-gray-100">
                        <div class="flex items-center justify-between mb-4">
                            <p class="text-xs font-medium text-gray-400 uppercase tracking-wider">
                                "Status"
                            </p>
                            <button
                                class="text-xs text-blue-600 hover:text-blue-700 cursor-pointer"
                                on:click=move |_| scrape_status.refetch()
                            >
                                "Refresh"
                            </button>
                        </div>
                        <Transition fallback=|| {
                            view! { <p class="text-sm text-gray-400">"Loading status…"</p> }
                        }>
                            {move || Suspend::new(async move {
                                match scrape_status.await {
                                    Ok(status) => {
                                        view! { <ScrapeStatusPanel status=status /> }.into_any()
                                    }
                                    Err(error) => {
                                        view! {
                                            <p class="text-sm text-red-600">{error.to_string()}</p>
                                        }
                                            .into_any()
                                    }
                                }
                            })}
                        </Transition>
                    </div>

                    <div class="px-6 py-5">
                        <Show when=move || matches!(trigger_scrape.value().get(), Some(Err(_)))>
                            <p class="text-sm text-red-500 mb-3">
                                {move || {
                                    trigger_scrape
                                        .value()
                                        .get()
                                        .and_then(|result| result.err())
                                        .map(|error| error.to_string())
                                        .unwrap_or_default()
                                }}
                            </p>
                        </Show>
                        <button
                            class="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
                            disabled=move || trigger_scrape.pending().get() || is_scrape_running()
                            on:click=move |_| {
                                trigger_scrape.dispatch(TriggerScrape {});
                            }
                        >
                            {move || {
                                if trigger_scrape.pending().get() {
                                    "Triggering…"
                                } else if is_scrape_running() {
                                    "Scrape Running…"
                                } else {
                                    "Run Scrape Now"
                                }
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </main>
    }
    .into_any()
}

#[component]
fn ScrapeStatusPanel(status: ScrapeStatus) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <div class="flex items-center gap-2">
                <span class=if status.is_running {
                    "inline-block w-2 h-2 rounded-full bg-yellow-400 animate-pulse"
                } else {
                    "inline-block w-2 h-2 rounded-full bg-green-400"
                } />
                <span class="text-sm font-medium text-gray-800">
                    {if status.is_running { "Running" } else { "Idle" }}
                </span>
            </div>
            {status
                .last_result_summary
                .map(|summary| {
                    view! { <StatusDetail label="Last result" value=summary /> }
                })}
            {status
                .last_run_at
                .map(|run_at| view! { <StatusDetail label="Last run" value=run_at /> })}
            {status
                .last_error
                .map(|error| {
                    view! {
                        <div>
                            <p class="text-xs text-red-400">"Last error"</p>
                            <p class="text-sm text-red-600">{error}</p>
                        </div>
                    }
                })}
        </div>
    }
    .into_any()
}

#[component]
fn StatusDetail(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div>
            <p class="text-xs text-gray-400">{label}</p>
            <p class="text-sm text-gray-600">{value}</p>
        </div>
    }
    .into_any()
}
