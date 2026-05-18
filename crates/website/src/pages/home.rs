use crate::components::team_picker::TeamPicker;
use crate::types::{Team, clear_tracked_team_id};
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    let all_teams = use_context::<RwSignal<Vec<Team>>>().expect("all_teams context");
    let tracked_team_id = use_context::<RwSignal<Option<i32>>>().expect("tracked_team_id context");
    let is_data_loaded = use_context::<RwSignal<bool>>().expect("is_data_loaded context");
    let is_copied = RwSignal::new(false);

    let tracked_team = Memo::new(move |_| -> Option<Team> {
        tracked_team_id.get().and_then(|team_id| {
            all_teams.get().into_iter().find(|team| team.team_id == team_id)
        })
    });

    let calendar_url = Memo::new(move |_| -> Option<String> {
        tracked_team_id
            .get()
            .map(|team_id| format!("https://calendar.footical.club/{}.ics", team_id))
    });

    let webcal_url = Memo::new(move |_| -> Option<String> {
        tracked_team_id
            .get()
            .map(|team_id| format!("webcal://calendar.footical.club/{}.ics", team_id))
    });

    let google_calendar_url = Memo::new(move |_| -> Option<String> {
        tracked_team_id.get().map(|team_id| {
            format!(
                "https://calendar.google.com/calendar/r?cid=webcal%3A%2F%2Fcalendar.footical.club%2F{}.ics",
                team_id
            )
        })
    });

    let on_change_team = move |_: leptos::ev::MouseEvent| {
        clear_tracked_team_id();
        tracked_team_id.set(None);
    };

    #[cfg(feature = "hydrate")]
    let on_copy_click = move |_: leptos::ev::MouseEvent| {
        if let Some(url) = calendar_url.get() {
            wasm_bindgen_futures::spawn_local(async move {
                if crate::clipboard::copy_to_clipboard(&url).await {
                    is_copied.set(true);
                    let reset_timeout = gloo_timers::callback::Timeout::new(1500, move || {
                        is_copied.set(false);
                    });
                    reset_timeout.forget();
                }
            });
        }
    };

    #[cfg(not(feature = "hydrate"))]
    let on_copy_click = move |_: leptos::ev::MouseEvent| {};

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-md">
                {move || {
                    if !is_data_loaded.get() {
                        return view! {
                            <div class="flex justify-center py-16">
                                <p class="text-sm text-gray-400">"Loading…"</p>
                            </div>
                        }.into_any();
                    }

                    if tracked_team.get().is_none() {
                        return view! {
                            <div class="bg-white rounded-xl shadow-md p-8 space-y-6">
                                <div>
                                    <h1 class="text-2xl font-bold text-gray-800">"Subscribe"</h1>
                                    <p class="text-sm text-gray-500 mt-1">
                                        "Search for your team to get subscription instructions."
                                    </p>
                                </div>
                                <TeamPicker />
                            </div>
                        }.into_any();
                    }

                    view! {
                        <div class="bg-white rounded-xl shadow-md overflow-hidden">
                            <div class="px-6 py-5 border-b border-gray-100 flex items-start justify-between">
                                <div>
                                    <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                                        "Subscribe"
                                    </p>
                                    <h1 class="text-xl font-bold text-gray-800 mt-0.5">
                                        {move || {
                                            tracked_team.get().map(|team| team.name).unwrap_or_default()
                                        }}
                                    </h1>
                                </div>
                                <button
                                    class="text-sm text-gray-400 hover:text-gray-600 transition-colors mt-0.5 cursor-pointer"
                                    on:click=on_change_team
                                >
                                    "Change team"
                                </button>
                            </div>

                            <div class="px-6 py-5 border-b border-gray-100">
                                <p class="text-xs font-medium text-gray-400 uppercase tracking-wider mb-2">
                                    "Your calendar link"
                                </p>
                                <div class="flex items-center gap-2">
                                    <span class="flex-1 text-sm text-gray-600 font-mono bg-gray-50 px-3 py-2 rounded-lg truncate">
                                        {move || calendar_url.get().unwrap_or_default()}
                                    </span>
                                    <button
                                        class="shrink-0 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-2 rounded-lg transition-colors cursor-pointer"
                                        on:click=on_copy_click
                                    >
                                        {move || if is_copied.get() { "Copied!" } else { "Copy" }}
                                    </button>
                                </div>
                            </div>

                            <div class="px-6 py-5 border-b border-gray-100 flex items-center justify-between gap-4">
                                <div>
                                    <p class="font-medium text-gray-800">"Apple Calendar"</p>
                                    <p class="text-sm text-gray-400 mt-0.5">
                                        "Opens directly on iPhone, iPad & Mac"
                                    </p>
                                </div>
                                <a
                                    href=move || webcal_url.get().unwrap_or_default()
                                    class="shrink-0 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-2 rounded-lg transition-colors cursor-pointer"
                                >
                                    "Subscribe"
                                </a>
                            </div>

                            <div class="px-6 py-5 border-b border-gray-100 flex items-center justify-between gap-4">
                                <div>
                                    <p class="font-medium text-gray-800">"Google Calendar"</p>
                                    <p class="text-sm text-gray-400 mt-0.5">
                                        "Opens in a new tab"
                                    </p>
                                </div>
                                <a
                                    href=move || google_calendar_url.get().unwrap_or_default()
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="shrink-0 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-2 rounded-lg transition-colors cursor-pointer"
                                >
                                    "Subscribe"
                                </a>
                            </div>

                            <div class="px-6 py-5">
                                <p class="font-medium text-gray-800 mb-2">"Outlook"</p>
                                <p class="text-sm text-gray-500 mb-3">
                                    "Copy the link above, then follow the steps for your version:"
                                </p>
                                <div class="space-y-3">
                                    <div>
                                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">
                                            "Web (Outlook.com or Office 365)"
                                        </p>
                                        <p class="text-sm text-gray-600">
                                            "Settings → View all Outlook settings → Calendar → Shared calendars → Subscribe from web → paste the link"
                                        </p>
                                    </div>
                                    <div>
                                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">
                                            "Desktop app"
                                        </p>
                                        <p class="text-sm text-gray-600">
                                            "File → Account Settings → Internet Calendars → New → paste the link"
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>
        </main>
    }
    .into_any()
}
