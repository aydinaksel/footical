use crate::components::team_picker::TeamPicker;
use crate::types::{Fixture, Team, clear_tracked_team_id};
use leptos::prelude::*;

#[component]
pub fn FixturesPage() -> impl IntoView {
    let all_teams = use_context::<RwSignal<Vec<Team>>>().expect("all_teams context");
    let all_fixtures = use_context::<RwSignal<Vec<Fixture>>>().expect("all_fixtures context");
    let tracked_team_id = use_context::<RwSignal<Option<i32>>>().expect("tracked_team_id context");
    let is_data_loading = use_context::<RwSignal<bool>>().expect("is_data_loading context");

    let tracked_team = Memo::new(move |_| -> Option<Team> {
        tracked_team_id.get().and_then(|team_id| {
            all_teams.get().into_iter().find(|team| team.team_id == team_id)
        })
    });

    let upcoming_fixtures = Memo::new(move |_| -> Vec<Fixture> {
        let Some(team_id) = tracked_team_id.get() else {
            return vec![];
        };
        let now = chrono::Utc::now().naive_utc();
        all_fixtures
            .get()
            .into_iter()
            .filter(|fixture| {
                (fixture.home_team_id == team_id || fixture.away_team_id == team_id)
                    && fixture.scheduled_at >= now
            })
            .collect()
    });

    let on_change_team = move |_: leptos::ev::MouseEvent| {
        clear_tracked_team_id();
        tracked_team_id.set(None);
    };

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-md">
                <Show
                    when=move || !is_data_loading.get()
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
                                            let date_label = fixture.scheduled_at.format("%a %-d %b").to_string();
                                            let time_label = fixture.scheduled_at.format("%H:%M").to_string();
                                            let is_not_scheduled = fixture.status != "scheduled";
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
