use chrono::NaiveDateTime;
use gloo_net::http::Request;
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

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct Fixture {
    fixture_id: i32,
    home_team_id: i32,
    away_team_id: i32,
    home_team_name: String,
    away_team_name: String,
    scheduled_at: String,
    status: String,
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
pub fn FixturesPage() -> impl IntoView {
    let all_leagues = RwSignal::new(Vec::<League>::new());
    let all_divisions = RwSignal::new(Vec::<Division>::new());
    let all_teams = RwSignal::new(Vec::<Team>::new());
    let all_fixtures = RwSignal::new(Vec::<Fixture>::new());
    let is_loading = RwSignal::new(true);

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

        let fixtures = Request::get("https://data.footical.club/fixtures.json")
            .send()
            .await
            .unwrap()
            .json::<Vec<Fixture>>()
            .await
            .unwrap_or_default();
        all_fixtures.set(fixtures);

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

    let upcoming_fixtures = Memo::new(move |_| -> Vec<Fixture> {
        let Some(team_id) = tracked_team_id.get() else {
            return vec![];
        };
        let now = chrono::Local::now().naive_local();
        all_fixtures
            .get()
            .into_iter()
            .filter(|fixture| {
                (fixture.home_team_id == team_id || fixture.away_team_id == team_id)
                    && NaiveDateTime::parse_from_str(&fixture.scheduled_at, "%Y-%m-%dT%H:%M:%S")
                        .map(|datetime| datetime >= now)
                        .unwrap_or(false)
            })
            .collect()
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
                                    <h1 class="text-2xl font-bold text-gray-800">"My Team"</h1>
                                    <p class="text-sm text-gray-500 mt-1">
                                        "Choose your team to follow their upcoming fixtures."
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
                            <div class="px-6 py-5 border-b border-gray-100 flex items-start justify-between">
                                <div>
                                    <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                                        "My Team"
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
                            <Show
                                when=move || !upcoming_fixtures.get().is_empty()
                                fallback=|| view! {
                                    <p class="text-sm text-gray-400 text-center py-12">
                                        "No upcoming fixtures."
                                    </p>
                                }
                            >
                                <ul class="divide-y divide-gray-100">
                                    <For
                                        each=move || upcoming_fixtures.get()
                                        key=|fixture| fixture.fixture_id
                                        children=move |fixture| {
                                            let team_id = tracked_team_id.get().unwrap_or(0);
                                            let is_home = fixture.home_team_id == team_id;
                                            let opponent = if is_home {
                                                fixture.away_team_name.clone()
                                            } else {
                                                fixture.home_team_name.clone()
                                            };
                                            let datetime = NaiveDateTime::parse_from_str(
                                                &fixture.scheduled_at,
                                                "%Y-%m-%dT%H:%M:%S",
                                            );
                                            let date_label = datetime
                                                .as_ref()
                                                .map(|datetime| datetime.format("%a %-d %b").to_string())
                                                .unwrap_or_default();
                                            let time_label = datetime
                                                .as_ref()
                                                .map(|datetime| datetime.format("%H:%M").to_string())
                                                .unwrap_or_default();
                                            let is_not_scheduled = !matches!(
                                                fixture.status.as_str(),
                                                "scheduled"
                                            );
                                            let status_label = fixture.status.to_uppercase();

                                            view! {
                                                <li class="px-6 py-4 flex items-center justify-between gap-4">
                                                    <div class="min-w-0">
                                                        <div class="flex items-center gap-2 mb-1">
                                                            <span class=if is_home {
                                                                "text-xs font-semibold text-blue-600 bg-blue-50 px-1.5 py-0.5 rounded"
                                                            } else {
                                                                "text-xs font-semibold text-gray-500 bg-gray-100 px-1.5 py-0.5 rounded"
                                                            }>
                                                                {if is_home { "HOME" } else { "AWAY" }}
                                                            </span>
                                                            <Show when=move || is_not_scheduled>
                                                                <span class="text-xs font-semibold text-amber-600 bg-amber-50 px-1.5 py-0.5 rounded">
                                                                    {status_label.clone()}
                                                                </span>
                                                            </Show>
                                                        </div>
                                                        <p class="font-medium text-gray-800 truncate">
                                                            {opponent}
                                                        </p>
                                                    </div>
                                                    <div class="text-right shrink-0">
                                                        <p class="font-medium text-gray-800">
                                                            {time_label}
                                                        </p>
                                                        <p class="text-sm text-gray-400">{date_label}</p>
                                                    </div>
                                                </li>
                                            }
                                        }
                                    />
                                </ul>
                            </Show>
                        </div>
                    </Show>
                </Show>
            </div>
        </main>
    }
}
