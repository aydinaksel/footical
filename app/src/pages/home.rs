use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct LeagueGroup {
    mundial_league_group_id: i32,
    name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct League {
    mundial_league_id: i32,
    mundial_league_group_id: i32,
    name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Team {
    mundial_team_id: i32,
    mundial_league_id: i32,
    name: String,
}

async fn fetch_league_groups() -> Vec<LeagueGroup> {
    Request::get("/mundial_league_groups.json")
        .send()
        .await
        .unwrap()
        .json::<Vec<LeagueGroup>>()
        .await
        .unwrap_or_default()
}

async fn fetch_leagues() -> Vec<League> {
    Request::get("/mundial_leagues.json")
        .send()
        .await
        .unwrap()
        .json::<Vec<League>>()
        .await
        .unwrap_or_default()
}

async fn fetch_teams() -> Vec<Team> {
    Request::get("/mundial_teams.json")
        .send()
        .await
        .unwrap()
        .json::<Vec<Team>>()
        .await
        .unwrap_or_default()
}

#[component]
pub fn Home() -> impl IntoView {
    let all_groups = RwSignal::new(Vec::<LeagueGroup>::new());
    let all_leagues = RwSignal::new(Vec::<League>::new());
    let all_teams = RwSignal::new(Vec::<Team>::new());

    let selected_group_id = RwSignal::new(Option::<i32>::None);
    let selected_league_id = RwSignal::new(Option::<i32>::None);
    let selected_team_id = RwSignal::new(Option::<i32>::None);
    let is_copied = RwSignal::new(false);

    spawn_local(async move {
        let loaded_groups = fetch_league_groups().await;
        all_groups.set(loaded_groups);
        let loaded_leagues = fetch_leagues().await;
        all_leagues.set(loaded_leagues);
        let loaded_teams = fetch_teams().await;
        all_teams.set(loaded_teams);
    });

    let filtered_leagues = Memo::new(move |_| -> Vec<League> {
        match selected_group_id.get() {
            Some(group_id) => all_leagues
                .get()
                .into_iter()
                .filter(|league| league.mundial_league_group_id == group_id)
                .collect(),
            None => vec![],
        }
    });

    let filtered_teams = Memo::new(move |_| -> Vec<Team> {
        match selected_league_id.get() {
            Some(league_id) => all_teams
                .get()
                .into_iter()
                .filter(|team| team.mundial_league_id == league_id)
                .collect(),
            None => vec![],
        }
    });

    let selected_team = Memo::new(move |_| -> Option<Team> {
        selected_team_id.get().and_then(|team_id| {
            all_teams
                .get()
                .into_iter()
                .find(|team| team.mundial_team_id == team_id)
        })
    });

    let calendar_url = Memo::new(move |_| -> Option<String> {
        selected_team_id.get().map(|team_id| {
            format!(
                "https://d39amfcda6iyyg.cloudfront.net/football_mundial/{}.ics",
                team_id
            )
        })
    });

    let on_group_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_group_id.set(value);
        selected_league_id.set(None);
        selected_team_id.set(None);
    };

    let on_league_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_league_id.set(value);
        selected_team_id.set(None);
    };

    let on_team_change = move |change_event: web_sys::Event| {
        let value = event_target_value(&change_event).parse::<i32>().ok();
        selected_team_id.set(value);
    };

    let on_copy_click = move |_: web_sys::MouseEvent| {
        if let Some(url) = calendar_url.get() {
            spawn_local(async move {
                let window = web_sys::window().expect("no window");
                let promise = window.navigator().clipboard().write_text(&url);
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                is_copied.set(true);
                let reset_timeout = Timeout::new(1500, move || {
                    is_copied.set(false);
                });
                reset_timeout.forget();
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
                            "League Group"
                        </label>
                        <select
                            class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=on_group_change
                        >
                            <option value="">"-- Select --"</option>
                            <For
                                each=move || all_groups.get()
                                key=|group| group.mundial_league_group_id
                                children=move |group| view! {
                                    <option value=group.mundial_league_group_id.to_string()>
                                        {group.name}
                                    </option>
                                }
                            />
                        </select>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "League"
                        </label>
                        <select
                            class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            prop:disabled=move || selected_group_id.get().is_none()
                            on:change=on_league_change
                        >
                            <option value="">"-- Select --"</option>
                            <For
                                each=move || filtered_leagues.get()
                                key=|league| league.mundial_league_id
                                children=move |league| view! {
                                    <option value=league.mundial_league_id.to_string()>
                                        {league.name}
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
                            prop:disabled=move || selected_league_id.get().is_none()
                            on:change=on_team_change
                        >
                            <option value="">"-- Select --"</option>
                            <For
                                each=move || filtered_teams.get()
                                key=|team| team.mundial_team_id
                                children=move |team| view! {
                                    <option value=team.mundial_team_id.to_string()>
                                        {team.name}
                                    </option>
                                }
                            />
                        </select>
                    </div>

                    <Show when=move || selected_team.get().is_some()>
                        <button
                            class="w-full bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-semibold py-2 px-4 rounded-lg transition-colors duration-150"
                            on:click=on_copy_click
                        >
                            {move || {
                                if is_copied.get() {
                                    "Copied!".to_string()
                                } else {
                                    format!(
                                        "Copy {} Calendar Link",
                                        selected_team.get().map(|team| team.name).unwrap_or_default()
                                    )
                                }
                            }}
                        </button>
                    </Show>
                </div>
            </div>
        </main>
    }
}
