use crate::server::auth::{check_auth, logout};
use crate::server::scraper::{ScrapeStatus, get_scrape_status, trigger_scrape};
use leptos::prelude::*;

#[component]
pub fn AdminPage() -> impl IntoView {
    let auth_resource = Resource::new(|| (), |_| check_auth());

    let status_signal = RwSignal::new(Option::<ScrapeStatus>::None);
    let is_triggering = RwSignal::new(false);
    let trigger_error = RwSignal::new(Option::<String>::None);

    Effect::new(move || {
        if let Some(Ok(true)) = auth_resource.get() {
            leptos::task::spawn_local(async move {
                if let Ok(status) = get_scrape_status().await {
                    status_signal.set(Some(status));
                }
            });
        }
    });

    let on_trigger = move |_: leptos::ev::MouseEvent| {
        is_triggering.set(true);
        trigger_error.set(None);

        leptos::task::spawn_local(async move {
            match trigger_scrape().await {
                Ok(()) => {
                    is_triggering.set(false);
                    if let Ok(status) = get_scrape_status().await {
                        status_signal.set(Some(status));
                    }
                }
                Err(error) => {
                    trigger_error.set(Some(error.to_string()));
                    is_triggering.set(false);
                }
            }
        });
    };

    let on_refresh = move |_: leptos::ev::MouseEvent| {
        leptos::task::spawn_local(async move {
            if let Ok(status) = get_scrape_status().await {
                status_signal.set(Some(status));
            }
        });
    };

    let on_logout = move |_: leptos::ev::MouseEvent| {
        leptos::task::spawn_local(async move {
            let _ = logout().await;
            let navigate = leptos_router::hooks::use_navigate();
            navigate("/admin/login", Default::default());
        });
    };

    let authenticated_view = move || {
        view! {
            <main class="flex justify-center p-4 pt-8">
                <div class="w-full max-w-lg">
                    <div class="bg-white rounded-xl shadow-md overflow-hidden">
                        <div class="px-6 py-5 border-b border-gray-100 flex items-center justify-between">
                            <div>
                                <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Admin"</p>
                                <h1 class="text-xl font-bold text-gray-800 mt-0.5">"Scraper Dashboard"</h1>
                            </div>
                            <button
                                class="text-sm text-gray-400 hover:text-gray-600 transition-colors cursor-pointer"
                                on:click=on_logout
                            >
                                "Logout"
                            </button>
                        </div>

                        <div class="px-6 py-5 border-b border-gray-100">
                            <div class="flex items-center justify-between mb-4">
                                <p class="text-xs font-medium text-gray-400 uppercase tracking-wider">"Status"</p>
                                <button
                                    class="text-xs text-blue-600 hover:text-blue-700 cursor-pointer"
                                    on:click=on_refresh
                                >
                                    "Refresh"
                                </button>
                            </div>
                            {move || {
                                match status_signal.get() {
                                    Some(status) => view! {
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
                                            <Show when={
                                                let summary = status.last_result_summary.clone();
                                                move || summary.is_some()
                                            }>
                                                <div>
                                                    <p class="text-xs text-gray-400">"Last result"</p>
                                                    <p class="text-sm text-gray-600">
                                                        {status.last_result_summary.clone().unwrap_or_default()}
                                                    </p>
                                                </div>
                                            </Show>
                                            <Show when={
                                                let run_at = status.last_run_at.clone();
                                                move || run_at.is_some()
                                            }>
                                                <div>
                                                    <p class="text-xs text-gray-400">"Last run"</p>
                                                    <p class="text-sm text-gray-600">
                                                        {status.last_run_at.clone().unwrap_or_default()}
                                                    </p>
                                                </div>
                                            </Show>
                                            <Show when={
                                                let error = status.last_error.clone();
                                                move || error.is_some()
                                            }>
                                                <div>
                                                    <p class="text-xs text-red-400">"Last error"</p>
                                                    <p class="text-sm text-red-600">
                                                        {status.last_error.clone().unwrap_or_default()}
                                                    </p>
                                                </div>
                                            </Show>
                                        </div>
                                    }.into_any(),
                                    None => view! {
                                        <p class="text-sm text-gray-400">"Loading status…"</p>
                                    }.into_any(),
                                }
                            }}
                        </div>

                        <div class="px-6 py-5">
                            <Show when=move || trigger_error.get().is_some()>
                                <p class="text-sm text-red-500 mb-3">
                                    {move || trigger_error.get().unwrap_or_default()}
                                </p>
                            </Show>
                            <button
                                class="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
                                disabled=move || is_triggering.get() || status_signal.get().map(|status| status.is_running).unwrap_or(false)
                                on:click=on_trigger
                            >
                                {move || {
                                    if is_triggering.get() {
                                        "Triggering…"
                                    } else if status_signal.get().map(|status| status.is_running).unwrap_or(false) {
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
        }.into_any()
    };

    let unauthenticated_redirect = move || {
        let navigate = leptos_router::hooks::use_navigate();
        navigate("/admin/login", Default::default());
        view! { <p>"Redirecting…"</p> }.into_any()
    };

    view! {
        {move || match auth_resource.get() {
            Some(Ok(true)) => authenticated_view(),
            Some(_) => unauthenticated_redirect(),
            None => view! {
                <div class="flex justify-center py-16">
                    <p class="text-sm text-gray-400">"Checking auth…"</p>
                </div>
            }.into_any(),
        }}
    }
    .into_any()
}
