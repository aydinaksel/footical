use crate::server::data::get_todays_fixtures;
use crate::types::TodayFixture;
use leptos::prelude::*;

#[component]
pub fn TodayPage() -> impl IntoView {
    let fixtures = Resource::new(|| (), |_| get_todays_fixtures());

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-2xl">
                <div class="mb-6">
                    <h1 class="text-2xl font-bold text-gray-800">"Today's Matches"</h1>
                    <p class="text-sm text-gray-500 mt-1">
                        {chrono::Local::now().format("%A %-d %B %Y").to_string()}
                    </p>
                </div>
                <Suspense fallback=move || view! {
                    <div class="flex justify-center py-16">
                        <p class="text-sm text-gray-400">"Loading..."</p>
                    </div>
                }>
                    {move || {
                        fixtures.get().map(|result| match result {
                            Err(_) => view! {
                                <p class="text-sm text-red-500 text-center py-12">
                                    "Failed to load fixtures."
                                </p>
                            }.into_any(),
                            Ok(fixtures) if fixtures.is_empty() => view! {
                                <p class="text-sm text-gray-400 text-center py-12">
                                    "No matches scheduled for today."
                                </p>
                            }.into_any(),
                            Ok(fixtures) => {
                                let grouped = group_by_league(fixtures);
                                view! {
                                    <div class="space-y-6">
                                        {grouped.into_iter().map(|(league_name, divisions)| view! {
                                            <div class="bg-white rounded-xl shadow-md overflow-hidden">
                                                <div class="px-6 py-4 bg-gray-50 border-b border-gray-100">
                                                    <h2 class="font-bold text-gray-800">{league_name}</h2>
                                                </div>
                                                {divisions.into_iter().map(|(division_name, venue, fixtures)| view! {
                                                    <div class="border-b border-gray-100 last:border-b-0">
                                                        <div class="px-6 py-3 flex items-baseline justify-between gap-4">
                                                            <p class="text-sm font-semibold text-gray-600">
                                                                {division_name}
                                                            </p>
                                                            {venue.map(|venue_text| view! {
                                                                <p class="text-xs text-gray-400 truncate">
                                                                    {venue_text}
                                                                </p>
                                                            })}
                                                        </div>
                                                        <ul class="divide-y divide-gray-50">
                                                            {fixtures.into_iter().map(|fixture| {
                                                                let time_label = fixture.scheduled_at.format("%H:%M").to_string();
                                                                let is_not_scheduled = fixture.status != "scheduled";
                                                                let status_label = fixture.status.to_uppercase();

                                                                view! {
                                                                    <li class="px-6 py-3 flex items-center justify-between gap-4">
                                                                        <div class="flex items-center gap-3 min-w-0">
                                                                            <span class="text-sm font-mono text-gray-400 shrink-0">
                                                                                {time_label}
                                                                            </span>
                                                                            <p class="text-sm text-gray-800 truncate">
                                                                                {fixture.home_team_name}
                                                                                " vs "
                                                                                {fixture.away_team_name}
                                                                            </p>
                                                                        </div>
                                                                        {is_not_scheduled.then(|| view! {
                                                                            <span class="text-xs font-semibold text-amber-600 bg-amber-50 px-1.5 py-0.5 rounded shrink-0">
                                                                                {status_label}
                                                                            </span>
                                                                        })}
                                                                    </li>
                                                                }
                                                            }).collect_view()}
                                                        </ul>
                                                    </div>
                                                }).collect_view()}
                                            </div>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </main>
    }
    .into_any()
}

fn group_by_league(
    fixtures: Vec<TodayFixture>,
) -> Vec<(String, Vec<(String, Option<String>, Vec<TodayFixture>)>)> {
    let mut leagues: Vec<(String, Vec<(String, Option<String>, Vec<TodayFixture>)>)> = Vec::new();

    for fixture in fixtures {
        let league_entry = leagues
            .iter_mut()
            .find(|(name, _)| *name == fixture.league_name);

        let venue_display = fixture
            .venue_name
            .clone()
            .or_else(|| fixture.venue_address.clone());

        if let Some((_, divisions)) = league_entry {
            let division_entry = divisions
                .iter_mut()
                .find(|(name, _, _)| *name == fixture.division_name);

            if let Some((_, _, division_fixtures)) = division_entry {
                division_fixtures.push(fixture);
            } else {
                let division_name = fixture.division_name.clone();
                divisions.push((division_name, venue_display, vec![fixture]));
            }
        } else {
            let league_name = fixture.league_name.clone();
            let division_name = fixture.division_name.clone();
            leagues.push((
                league_name,
                vec![(division_name, venue_display, vec![fixture])],
            ));
        }
    }

    leagues
}
