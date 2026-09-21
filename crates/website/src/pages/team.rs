use crate::server::squad::{get_player_balances, get_squad_fixtures};
use crate::types::{format_pence, PlayerBalance, SquadFixture};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

const TEAM_NAME: &str = "Wigginton Grasshoppers Reserves";

#[component]
pub fn TeamLayout(children: ChildrenFn) -> impl IntoView {
    let location = use_location();

    let tab_class = move |path: &'static str| {
        move || {
            if location.pathname.get() == path {
                "pb-2 border-b-2 border-blue-600 text-blue-600 font-medium text-sm"
            } else {
                "pb-2 border-b-2 border-transparent text-gray-500 hover:text-gray-800 text-sm"
            }
        }
    };

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-2xl space-y-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800">{TEAM_NAME}</h1>
                    <p class="text-sm text-gray-500 mt-1">"York Football League"</p>
                </div>

                <nav class="flex gap-6 border-b border-gray-200">
                    <A href="/team/fixtures">
                        <span class=tab_class("/team/fixtures")>"Fixtures"</span>
                    </A>
                    <A href="/team/fines">
                        <span class=tab_class("/team/fines")>"Fines"</span>
                    </A>
                </nav>

                {children()}
            </div>
        </main>
    }
    .into_any()
}

#[component]
pub fn TeamFixturesPage() -> impl IntoView {
    let fixtures = Resource::new_blocking(|| (), |_| get_squad_fixtures());

    view! {
        <TeamLayout>
            <Suspense fallback=move || {
                view! {
                    <p class="text-sm text-gray-400 py-8 text-center">"Loading fixtures…"</p>
                }
            }>
                {move || {
                    fixtures
                        .get()
                        .map(|result| match result {
                            Err(_) => {
                                view! {
                                    <p class="text-sm text-red-500 py-8 text-center">
                                        "Failed to load fixtures."
                                    </p>
                                }
                                    .into_any()
                            }
                            Ok(fixtures) => fixture_list(fixtures),
                        })
                }}
            </Suspense>
        </TeamLayout>
    }
    .into_any()
}

#[component]
pub fn TeamFinesPage() -> impl IntoView {
    let balances = Resource::new_blocking(|| (), |_| get_player_balances());

    view! {
        <TeamLayout>
            <Suspense fallback=move || {
                view! { <p class="text-sm text-gray-400 py-8 text-center">"Loading fines…"</p> }
            }>
                {move || {
                    balances
                        .get()
                        .map(|result| match result {
                            Err(_) => {
                                view! {
                                    <p class="text-sm text-red-500 py-8 text-center">
                                        "Failed to load fines."
                                    </p>
                                }
                                    .into_any()
                            }
                            Ok(balances) => fines_table(balances),
                        })
                }}
            </Suspense>
        </TeamLayout>
    }
    .into_any()
}

fn fixture_list(fixtures: Vec<SquadFixture>) -> AnyView {
    if fixtures.is_empty() {
        return view! {
            <div class="bg-white rounded-xl shadow-md p-8">
                <p class="text-sm text-gray-400 text-center">"No upcoming fixtures."</p>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="bg-white rounded-xl shadow-md overflow-hidden">
            <ul class="divide-y divide-gray-100">
                {fixtures
                    .into_iter()
                    .map(|fixture| {
                        let date_label = fixture.kicks_off_at.format("%a %-d %b").to_string();
                        let time_label = fixture.kicks_off_at.format("%H:%M").to_string();
                        let venue = fixture.venue.unwrap_or_default();
                        let is_home = fixture.is_home;
                        let competition = fixture.competition;
                        let opponent = fixture.opponent;

                        view! {
                            <li class="px-6 py-4 flex items-start justify-between gap-4">
                                <div class="min-w-0">
                                    <div class="flex items-center gap-2 mb-1">
                                        <span class=if is_home {
                                            "text-xs font-semibold text-blue-600 bg-blue-50 px-1.5 py-0.5 rounded"
                                        } else {
                                            "text-xs font-semibold text-gray-500 bg-gray-100 px-1.5 py-0.5 rounded"
                                        }>{if is_home { "HOME" } else { "AWAY" }}</span>
                                        <span class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                                            {competition}
                                        </span>
                                    </div>
                                    <p class="font-medium text-gray-800 truncate">{opponent}</p>
                                    <p class="text-xs text-gray-400 truncate mt-0.5">{venue}</p>
                                </div>
                                <div class="text-right shrink-0">
                                    <p class="font-medium text-gray-800">{time_label}</p>
                                    <p class="text-sm text-gray-400">{date_label}</p>
                                </div>
                            </li>
                        }
                            .into_any()
                    })
                    .collect_view()}
            </ul>
        </div>
    }
    .into_any()
}

fn fines_table(balances: Vec<PlayerBalance>) -> AnyView {
    let total_fines: i64 = balances.iter().map(|balance| balance.fines_pence).sum();
    let total_payments: i64 = balances.iter().map(|balance| balance.payments_pence).sum();
    let total_outstanding = total_fines.saturating_sub(total_payments);

    view! {
        <div class="bg-white rounded-xl shadow-md overflow-hidden">
            <div class="px-6 py-4 bg-gray-50 border-b border-gray-100 flex items-baseline justify-between gap-4">
                <h2 class="font-bold text-gray-800">"Fines"</h2>
                <p class="text-sm text-gray-500">
                    {format!("{} outstanding", format_pence(total_outstanding))}
                </p>
            </div>
            <div class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                            <th class="text-left px-6 py-3">"Name"</th>
                            <th class="text-right px-3 py-3">"Fines"</th>
                            <th class="text-right px-3 py-3">"Paid"</th>
                            <th class="text-right px-6 py-3">"Balance"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-50">
                        {balances
                            .into_iter()
                            .map(|balance| {
                                let owes = balance.balance_pence > 0;
                                let is_settled = balance.fines_pence == 0;
                                view! {
                                    <tr>
                                        <td class="px-6 py-2.5">
                                            <a
                                                href=format!("/team/player/{}", balance.squad_player_id)
                                                class="text-gray-800 hover:text-blue-600"
                                            >
                                                {balance.name}
                                            </a>
                                        </td>
                                        <td class="px-3 py-2.5 text-right text-gray-500 font-mono">
                                            {format_pence(balance.fines_pence)}
                                        </td>
                                        <td class="px-3 py-2.5 text-right text-gray-500 font-mono">
                                            {format_pence(balance.payments_pence)}
                                        </td>
                                        <td class=move || {
                                            if owes {
                                                "px-6 py-2.5 text-right font-mono font-semibold text-red-600"
                                            } else if is_settled {
                                                "px-6 py-2.5 text-right font-mono text-gray-300"
                                            } else {
                                                "px-6 py-2.5 text-right font-mono text-green-600"
                                            }
                                        }>{format_pence(balance.balance_pence)}</td>
                                    </tr>
                                }
                                    .into_any()
                            })
                            .collect_view()}
                    </tbody>
                    <tfoot class="border-t border-gray-200">
                        <tr class="font-semibold text-gray-800">
                            <td class="px-6 py-3">"Total"</td>
                            <td class="px-3 py-3 text-right font-mono">
                                {format_pence(total_fines)}
                            </td>
                            <td class="px-3 py-3 text-right font-mono">
                                {format_pence(total_payments)}
                            </td>
                            <td class="px-6 py-3 text-right font-mono">
                                {format_pence(total_outstanding)}
                            </td>
                        </tr>
                    </tfoot>
                </table>
            </div>
        </div>
    }
    .into_any()
}
