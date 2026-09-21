use crate::components::team_picker::TeamPicker;
use crate::server::data::{get_team_fixtures, get_team_listing};
use crate::tracked_team::{use_tracked_team, TrackedTeam};
use crate::types::{Fixture, TeamListing};
use leptos::prelude::*;

#[component]
pub fn FixturesPage() -> impl IntoView {
    let TrackedTeam {
        team_id: tracked_team_id,
        ..
    } = use_tracked_team();

    let tracked_team = Resource::new_blocking(
        move || tracked_team_id.get(),
        |team_id| async move {
            match team_id {
                Some(team_id) => get_team_listing(team_id).await,
                None => Ok(None),
            }
        },
    );

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-md">
                <Suspense fallback=|| ()>
                    {move || Suspend::new(async move {
                        match tracked_team.await {
                            Ok(Some(team)) => view! { <TeamFixtures team=team /> }.into_any(),
                            Ok(None) => view! { <TeamPrompt /> }.into_any(),
                            Err(_) => view! { <FixturesUnavailable /> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </main>
    }
    .into_any()
}

#[component]
fn TeamPrompt() -> impl IntoView {
    view! {
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
    .into_any()
}

#[component]
fn FixturesUnavailable() -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl shadow-md p-8">
            <h1 class="text-2xl font-bold text-gray-800">"My Team"</h1>
            <p class="text-sm text-gray-500 mt-1">
                "Your fixtures could not be loaded. Reload the page to try again."
            </p>
        </div>
    }
    .into_any()
}

#[component]
fn TeamFixtures(team: TeamListing) -> impl IntoView {
    let TrackedTeam { set_team_id, .. } = use_tracked_team();
    let team_id = team.team_id;
    let upcoming_fixtures = Resource::new_blocking(move || team_id, get_team_fixtures);

    view! {
        <div class="bg-white rounded-xl shadow-md overflow-hidden">
            <div class="px-6 py-5 border-b border-gray-100 flex items-start justify-between">
                <div>
                    <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                        "My Team"
                    </p>
                    <h1 class="text-xl font-bold text-gray-800 mt-0.5">{team.team_name}</h1>
                </div>
                <button
                    class="text-sm text-gray-400 hover:text-gray-600 transition-colors mt-0.5 cursor-pointer"
                    on:click=move |_| set_team_id.set(None)
                >
                    "Change team"
                </button>
            </div>
            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    let fixtures = upcoming_fixtures.await.unwrap_or_default();
                    if fixtures.is_empty() {
                        return view! {
                            <p class="text-sm text-gray-400 text-center py-12">
                                "No upcoming fixtures."
                            </p>
                        }
                            .into_any();
                    }

                    view! {
                        <ul class="divide-y divide-gray-100">
                            <For
                                each=move || fixtures.clone()
                                key=|fixture| fixture.fixture_id
                                children=move |fixture| {
                                    view! { <FixtureRow fixture=fixture tracked_team_id=team_id /> }
                                }
                            />
                        </ul>
                    }
                        .into_any()
                })}
            </Suspense>
        </div>
    }
    .into_any()
}

#[component]
fn FixtureRow(fixture: Fixture, tracked_team_id: i32) -> impl IntoView {
    let is_home = fixture.home_team_id == tracked_team_id;
    let opponent = if is_home {
        fixture.away_team_name
    } else {
        fixture.home_team_name
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
                    }>{if is_home { "HOME" } else { "AWAY" }}</span>
                    {if is_not_scheduled {
                        Some(
                            view! {
                                <span class="text-xs font-semibold text-amber-600 bg-amber-50 px-1.5 py-0.5 rounded">
                                    {status_label}
                                </span>
                            },
                        )
                    } else {
                        None
                    }}
                </div>
                <p class="font-medium text-gray-800 truncate">{opponent}</p>
            </div>
            <div class="text-right shrink-0">
                <p class="font-medium text-gray-800">{time_label}</p>
                <p class="text-sm text-gray-400">{date_label}</p>
            </div>
        </li>
    }
    .into_any()
}
