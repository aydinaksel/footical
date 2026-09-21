#![expect(
    clippy::mem_forget,
    reason = "the island macro forgets the render state it built on the server"
)]

use crate::components::admin_only::AdminOnly;
use crate::components::status_message::{StatusLine, StatusMessage};
use crate::server::squad::{get_squad_roster, SetPlayerActive};
use crate::types::{format_pence, SquadRosterEntry};
use leptos::prelude::*;

#[component]
pub fn SquadAdminPage() -> impl IntoView {
    view! {
        <AdminOnly>
            <SquadRoster />
        </AdminOnly>
    }
    .into_any()
}

#[island]
fn SquadRoster() -> impl IntoView {
    let status = RwSignal::new(Option::<StatusMessage>::None);
    let set_player_active = ServerAction::<SetPlayerActive>::new();
    let roster = Resource::new(
        move || set_player_active.version().get(),
        |_| get_squad_roster(),
    );

    let last_toggle_made_active = RwSignal::new(false);

    Effect::new(move |_| match set_player_active.value().get() {
        Some(Ok(())) => {
            status.set(Some(StatusMessage::confirmation(
                if last_toggle_made_active.get_untracked() {
                    "Player shown"
                } else {
                    "Player hidden"
                },
            )));
        }
        Some(Err(error)) => status.set(Some(StatusMessage::failure(error.to_string()))),
        None => {}
    });

    let on_toggle = move |squad_player_id: i32, make_active: bool| {
        last_toggle_made_active.set(make_active);
        set_player_active.dispatch(SetPlayerActive {
            squad_player_id,
            is_active: make_active,
        });
    };

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-lg space-y-6">
                <div class="flex items-baseline justify-between">
                    <div>
                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                            "Admin"
                        </p>
                        <h1 class="text-xl font-bold text-gray-800 mt-0.5">"Squad"</h1>
                    </div>
                    <a href="/admin/fines" class="text-sm text-blue-600 hover:text-blue-700">
                        "Fines"
                    </a>
                </div>

                <p class="text-sm text-gray-500">
                    "Hidden players stay out of the fines table and the admin dropdowns. \
                     Their fines and payments are kept."
                </p>

                <StatusLine status=status />

                <Transition fallback=move || {
                    view! {
                        <p class="text-sm text-gray-400 text-center py-8">"Loading squad…"</p>
                    }
                }>
                    {move || Suspend::new(async move {
                        let players = roster.await.unwrap_or_default();
                        roster_view(
                            players,
                            Callback::new(move |(id, active)| { on_toggle(id, active) }),
                        )
                    })}
                </Transition>
            </div>
        </main>
    }
    .into_any()
}

fn roster_view(players: Vec<SquadRosterEntry>, on_toggle: Callback<(i32, bool)>) -> AnyView {
    let active_count = players.iter().filter(|player| player.is_active).count();
    let hidden_count = players.len().saturating_sub(active_count);
    let hidden_owing: i64 = players
        .iter()
        .filter(|player| !player.is_active && player.balance_pence > 0)
        .map(|player| player.balance_pence)
        .sum();

    view! {
        <div class="bg-white rounded-xl shadow-md overflow-hidden">
            <div class="px-6 py-4 bg-gray-50 border-b border-gray-100 flex items-baseline justify-between gap-4">
                <h2 class="font-bold text-gray-800">{format!("{active_count} shown")}</h2>
                <p class="text-sm text-gray-500">{format!("{hidden_count} hidden")}</p>
            </div>

            {(hidden_owing > 0)
                .then(|| {
                    view! {
                        <p class="px-6 py-3 text-sm text-amber-700 bg-amber-50 border-b border-amber-100">
                            {format!(
                                "{} is owed by hidden players and is not counted in the fines total.",
                                format_pence(hidden_owing),
                            )}
                        </p>
                    }
                        .into_any()
                })}

            <ul class="divide-y divide-gray-50">
                {players
                    .into_iter()
                    .map(|player| {
                        let squad_player_id = player.squad_player_id;
                        let is_active = player.is_active;
                        let owes = player.balance_pence > 0;
                        view! {
                            <li class="px-6 py-2.5 flex items-center justify-between gap-3">
                                <div class="min-w-0">
                                    <p class=if is_active {
                                        "text-sm text-gray-800 truncate"
                                    } else {
                                        "text-sm text-gray-400 truncate"
                                    }>{player.name}</p>
                                </div>
                                <div class="flex items-center gap-3 shrink-0">
                                    <span class=if owes {
                                        "font-mono text-sm text-red-600"
                                    } else {
                                        "font-mono text-sm text-gray-300"
                                    }>{format_pence(player.balance_pence)}</span>
                                    <button
                                        type="button"
                                        class=if is_active {
                                            "text-xs text-gray-400 hover:text-gray-700 cursor-pointer w-10 text-right"
                                        } else {
                                            "text-xs text-blue-600 hover:text-blue-700 cursor-pointer w-10 text-right"
                                        }
                                        on:click=move |_| {
                                            on_toggle.run((squad_player_id, !is_active))
                                        }
                                    >
                                        {if is_active { "Hide" } else { "Show" }}
                                    </button>
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
