use crate::clipboard::copy_to_clipboard;
use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct League {
    league_id: i32,
    name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Division {
    division_id: i32,
    league_id: i32,
    name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Team {
    team_id: i32,
    division_id: i32,
    name: String,
}

fn read_tracked_team_id() -> Option<i32> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item("footical_team_id")
        .ok()?
        .and_then(|value| value.parse::<i32>().ok())
}

fn save_tracked_team_id(team_id: i32) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("footical_team_id", &team_id.to_string());
    }
}

fn clear_tracked_team_id() {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.remove_item("footical_team_id");
    }
}

#[component]
pub fn SubscribePage() -> impl IntoView {
    let all_leagues = RwSignal::new(Vec::<League>::new());
    let all_divisions = RwSignal::new(Vec::<Division>::new());
    let all_teams = RwSignal::new(Vec::<Team>::new());
    let is_loading = RwSignal::new(true);
    let is_copied = RwSignal::new(false);

    let selected_league_id = RwSignal::new(Option::<i32>::None);
    let selected_division_id = RwSignal::new(Option::<i32>::None);
    let tracked_team_id = RwSignal::new(read_tracked_team_id());

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

        is_loading.set(false);
    });

    let filtered_divisions = Memo::new(move |_| -> Vec<Division> {
        match selected_league_id.get() {
            Some(league_id) => all_divisions
                .get()
                .into_iter()
                .filter(|division| division.league_id == league_id)
                .collect(),
            None => vec![],
        }
    });

    let filtered_teams = Memo::new(move |_| -> Vec<Team> {
        match selected_division_id.get() {
            Some(division_id) => all_teams
                .get()
                .into_iter()
                .filter(|team| team.division_id == division_id)
                .collect(),
            None => vec![],
        }
    });

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

    let on_league_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_league_id.set(value);
        selected_division_id.set(None);
    };

    let on_division_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_division_id.set(value);
    };

    let on_team_change = move |change_event: web_sys::Event| {
        if let Some(team_id) = event_target_value(&change_event).parse::<i32>().ok() {
            save_tracked_team_id(team_id);
            tracked_team_id.set(Some(team_id));
        }
    };

    let on_change_team = move |_: web_sys::MouseEvent| {
        clear_tracked_team_id();
        tracked_team_id.set(None);
        selected_league_id.set(None);
        selected_division_id.set(None);
    };

    let on_copy_click = move |_: web_sys::MouseEvent| {
        if let Some(url) = calendar_url.get() {
            spawn_local(async move {
                if copy_to_clipboard(&url).await {
                    is_copied.set(true);
                    let reset_timeout = Timeout::new(1500, move || {
                        is_copied.set(false);
                    });
                    reset_timeout.forget();
                }
            });
        }
    };

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-md">
                <Show
                    when=move || !is_loading.get()
                    fallback=|| view! {
                        <div class="flex justify-center py-16">
                            <p class="text-sm text-gray-400">"Loading…"</p>
                        </div>
                    }
                >
                    <Show
                        when=move || tracked_team.get().is_some()
                        fallback=move || view! {
                            <div class="bg-white rounded-xl shadow-md p-8 space-y-6">
                                <div>
                                    <h1 class="text-2xl font-bold text-gray-800">"Subscribe"</h1>
                                    <p class="text-sm text-gray-500 mt-1">
                                        "Choose your team to get subscription instructions."
                                    </p>
                                </div>
                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "League"
                                        </label>
                                        <select
                                            class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            on:change=on_league_change
                                        >
                                            <option value="">"-- Select --"</option>
                                            <For
                                                each=move || all_leagues.get()
                                                key=|league| league.league_id
                                                children=move |league| view! {
                                                    <option value=league.league_id.to_string()>
                                                        {league.name}
                                                    </option>
                                                }
                                            />
                                        </select>
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "Division"
                                        </label>
                                        <select
                                            class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                                            prop:disabled=move || selected_league_id.get().is_none()
                                            on:change=on_division_change
                                        >
                                            <option value="">"-- Select --"</option>
                                            <For
                                                each=move || filtered_divisions.get()
                                                key=|division| division.division_id
                                                children=move |division| view! {
                                                    <option value=division.division_id.to_string()>
                                                        {division.name}
                                                    </option>
                                                }
                                            />
                                        </select>
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "Team"
                                        </label>
                                        <select
                                            class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                                            prop:disabled=move || selected_division_id.get().is_none()
                                            on:change=on_team_change
                                        >
                                            <option value="">"-- Select --"</option>
                                            <For
                                                each=move || filtered_teams.get()
                                                key=|team| team.team_id
                                                children=move |team| view! {
                                                    <option value=team.team_id.to_string()>
                                                        {team.name}
                                                    </option>
                                                }
                                            />
                                        </select>
                                    </div>
                                </div>
                            </div>
                        }
                    >
                        <div class="bg-white rounded-xl shadow-md overflow-hidden">
                            // Header
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

                            // Calendar link
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

                            // Apple Calendar
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

                            // Google Calendar
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

                            // Outlook
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
                    </Show>
                </Show>
            </div>
        </main>
    }
}
