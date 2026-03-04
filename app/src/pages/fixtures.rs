use chrono::NaiveDateTime;
use crate::components::team_picker::TeamPicker;
use crate::types::{Team, clear_tracked_team_id};
use gloo_net::http::Request;
use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

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

#[component]
pub fn FixturesPage() -> impl IntoView {
    let all_teams = use_context::<RwSignal<Vec<Team>>>().expect("all_teams context");
    let tracked_team_id = use_context::<RwSignal<Option<i32>>>().expect("tracked_team_id context");
    let is_data_loading = use_context::<RwSignal<bool>>().expect("is_data_loading context");

    let all_fixtures = RwSignal::new(Vec::<Fixture>::new());
    let is_fixtures_loading = RwSignal::new(true);

    spawn_local(async move {
        let fixtures = Request::get("https://data.footical.club/fixtures.json")
            .send()
            .await
            .unwrap()
            .json::<Vec<Fixture>>()
            .await
            .unwrap_or_default();
        all_fixtures.set(fixtures);
        is_fixtures_loading.set(false);
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

    let on_change_team = move |_: web_sys::MouseEvent| {
        clear_tracked_team_id();
        tracked_team_id.set(None);
    };

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-md">
                <Show
                    when=move || !is_data_loading.get() && !is_fixtures_loading.get()
                    fallback=|| view! {
                        <div class="flex justify-center py-16">
                            <p class="text-sm text-gray-400">"Loading…"</p>
                        </div>
                    }
                >
                    <Show
                        when=move || tracked_team.get().is_some()
                        fallback=|| view! {
                            <div class="bg-white rounded-xl shadow-md p-8 space-y-6">
                                <div>
                                    <h1 class="text-2xl font-bold text-gray-800">"My Team"</h1>
                                    <p class="text-sm text-gray-500 mt-1">
                                        "Search for your team to follow their upcoming fixtures."
                                    </p>
                                </div>
                                <TeamPicker />
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
