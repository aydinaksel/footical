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

async fn fetch_leagues() -> Vec<League> {
    Request::get("https://data.footical.club/leagues.json")
        .send()
        .await
        .unwrap()
        .json::<Vec<League>>()
        .await
        .unwrap_or_default()
}

async fn fetch_divisions() -> Vec<Division> {
    Request::get("https://data.footical.club/divisions.json")
        .send()
        .await
        .unwrap()
        .json::<Vec<Division>>()
        .await
        .unwrap_or_default()
}

async fn fetch_teams() -> Vec<Team> {
    Request::get("https://data.footical.club/teams.json")
        .send()
        .await
        .unwrap()
        .json::<Vec<Team>>()
        .await
        .unwrap_or_default()
}

#[component]
pub fn Home() -> impl IntoView {
    let all_leagues = RwSignal::new(Vec::<League>::new());
    let all_divisions = RwSignal::new(Vec::<Division>::new());
    let all_teams = RwSignal::new(Vec::<Team>::new());

    let selected_league_id = RwSignal::new(Option::<i32>::None);
    let selected_division_id = RwSignal::new(Option::<i32>::None);
    let selected_team_id = RwSignal::new(Option::<i32>::None);
    let is_copied = RwSignal::new(false);

    spawn_local(async move {
        let loaded_leagues = fetch_leagues().await;
        all_leagues.set(loaded_leagues);
        let loaded_divisions = fetch_divisions().await;
        all_divisions.set(loaded_divisions);
        let loaded_teams = fetch_teams().await;
        all_teams.set(loaded_teams);
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

    let selected_team = Memo::new(move |_| -> Option<Team> {
        selected_team_id.get().and_then(|team_id| {
            all_teams
                .get()
                .into_iter()
                .find(|team| team.team_id == team_id)
        })
    });

    let calendar_url = Memo::new(move |_| -> Option<String> {
        selected_team_id
            .get()
            .map(|team_id| format!("https://calendar.footical.club/{}.ics", team_id))
    });

    let on_league_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_league_id.set(value);
        selected_division_id.set(None);
        selected_team_id.set(None);
    };

    let on_division_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_division_id.set(value);
        selected_team_id.set(None);
    };

    let on_team_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_team_id.set(value);
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
            <div class="bg-white rounded-xl shadow-md p-8 w-full max-w-md space-y-6">
                <h1 class="text-2xl font-bold text-gray-800">"iCal Link Generator"</h1>

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

                    <Show when=move || selected_team.get().is_some()>
                        <button
                            class="w-full bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-semibold py-2 px-4 rounded-lg transition-colors duration-150 cursor-pointer"
                            on:click=on_copy_click
                        >
                            {move || if is_copied.get() { "Copied!" } else { "Copy Calendar Link" }}
                        </button>
                    </Show>
                </div>
            </div>
        </main>
    }
}
