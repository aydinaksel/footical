use crate::server::squad::{get_player_balances, get_player_ledger};
use crate::types::{format_pence, LedgerEntry, PlayerBalance};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn PlayerPage() -> impl IntoView {
    let params = use_params_map();
    let player_id = move || {
        params
            .read()
            .get("id")
            .and_then(|value| value.parse::<i32>().ok())
    };

    let ledger = Resource::new(player_id, |id| async move {
        match id {
            Some(id) => get_player_ledger(id).await,
            None => Ok(Vec::new()),
        }
    });
    let balances = Resource::new(|| (), |_| get_player_balances());

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-lg space-y-6">
                <a
                    href="/team/fines"
                    class="text-sm text-blue-600 hover:text-blue-700 inline-block"
                >
                    "← Back to team"
                </a>

                <Suspense fallback=move || {
                    view! { <p class="text-sm text-gray-400 text-center py-16">"Loading…"</p> }
                }>
                    {move || {
                        let entries = ledger.get().and_then(|result| result.ok())?;
                        let all_balances = balances.get().and_then(|result| result.ok())?;
                        let balance = player_id()
                            .and_then(|id| {
                                all_balances
                                    .into_iter()
                                    .find(|balance| balance.squad_player_id == id)
                            });
                        Some(
                            match balance {
                                Some(balance) => player_view(balance, entries),
                                None => {
                                    view! {
                                        <p class="text-sm text-gray-400 text-center py-16">
                                            "Player not found."
                                        </p>
                                    }
                                        .into_any()
                                }
                            },
                        )
                    }}
                </Suspense>
            </div>
        </main>
    }
    .into_any()
}

fn player_view(balance: PlayerBalance, entries: Vec<LedgerEntry>) -> AnyView {
    let owes = balance.balance_pence > 0;

    view! {
        <div class="bg-white rounded-xl shadow-md overflow-hidden">
            <div class="px-6 py-5 border-b border-gray-100">
                <h1 class="text-xl font-bold text-gray-800">{balance.name}</h1>
                <div class="flex gap-6 mt-3">
                    <div>
                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                            "Fines"
                        </p>
                        <p class="font-mono text-gray-800">{format_pence(balance.fines_pence)}</p>
                    </div>
                    <div>
                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                            "Paid"
                        </p>
                        <p class="font-mono text-gray-800">
                            {format_pence(balance.payments_pence)}
                        </p>
                    </div>
                    <div>
                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                            "Balance"
                        </p>
                        <p class=if owes {
                            "font-mono font-semibold text-red-600"
                        } else {
                            "font-mono text-gray-400"
                        }>{format_pence(balance.balance_pence)}</p>
                    </div>
                </div>
            </div>

            {if entries.is_empty() {
                view! {
                    <p class="text-sm text-gray-400 text-center py-12">"Nothing recorded yet."</p>
                }
                    .into_any()
            } else {
                view! {
                    <ul class="divide-y divide-gray-50">
                        {entries.into_iter().map(entry_row).collect_view()}
                    </ul>
                }
                    .into_any()
            }}
        </div>
    }
    .into_any()
}

fn entry_row(entry: LedgerEntry) -> AnyView {
    let is_payment = entry.is_payment;
    let note = entry.note.unwrap_or_default();

    view! {
        <li class="px-6 py-3 flex items-center justify-between gap-4">
            <div class="min-w-0">
                <p class="text-sm text-gray-800 truncate">{entry.description}</p>
                <p class="text-xs text-gray-400 mt-0.5">
                    {entry.happened_on}
                    {if note.is_empty() { String::new() } else { format!(" · {note}") }}
                </p>
            </div>
            <p class=if is_payment {
                "font-mono text-sm text-green-600 shrink-0"
            } else {
                "font-mono text-sm text-gray-800 shrink-0"
            }>
                {if is_payment {
                    format!("-{}", format_pence(entry.amount_pence))
                } else {
                    format_pence(entry.amount_pence)
                }}
            </p>
        </li>
    }
    .into_any()
}
