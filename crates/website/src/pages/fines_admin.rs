use crate::server::auth::check_auth;
use crate::server::squad::{
    delete_entry, get_fine_types, get_recent_entries, get_squad_players, record_fine,
    record_payment,
};
use crate::components::searchable_select::{SearchableSelect, SelectOption};
use crate::components::toast::use_toaster;
use crate::types::{FineType, SquadPlayer, format_pence};
use leptos::prelude::*;

fn parse_pounds_to_pence(text: &str) -> Option<i64> {
    let trimmed = text.trim().trim_start_matches('£').trim();
    if !trimmed.chars().any(|character| character.is_ascii_digit()) {
        return None;
    }
    let (pounds_text, pence_text) = match trimmed.split_once('.') {
        Some((pounds, pence)) => (pounds, pence),
        None => (trimmed, ""),
    };
    let pounds = if pounds_text.is_empty() {
        0
    } else {
        pounds_text.parse::<i64>().ok()?
    };
    let pence = match pence_text.len() {
        0 => 0,
        1 => pence_text.parse::<i64>().ok()?.checked_mul(10)?,
        2 => pence_text.parse::<i64>().ok()?,
        _ => return None,
    };
    if pounds < 0 || pence < 0 {
        return None;
    }
    pounds.checked_mul(100)?.checked_add(pence)
}

#[component]
pub fn FinesAdminPage() -> impl IntoView {
    let auth_resource = Resource::new(|| (), |_| check_auth());

    view! {
        {move || match auth_resource.get() {
            Some(Ok(true)) => view! { <FinesAdminForms /> }.into_any(),
            Some(_) => view! {
                <leptos_router::components::Redirect path="/admin/login" />
            }
            .into_any(),
            None => view! {
                <p class="text-sm text-gray-400 text-center py-16">"Checking auth…"</p>
            }
            .into_any(),
        }}
    }
    .into_any()
}

#[component]
fn FinesAdminForms() -> impl IntoView {
    let players = Resource::new(|| (), |_| get_squad_players());
    let fine_types = Resource::new(|| (), |_| get_fine_types());
    let ledger_version = RwSignal::new(0_u32);

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-lg space-y-6">
                <div class="flex items-baseline justify-between">
                    <div>
                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                            "Admin"
                        </p>
                        <h1 class="text-xl font-bold text-gray-800 mt-0.5">"Fines & Payments"</h1>
                    </div>
                    <a href="/team/fines" class="text-sm text-blue-600 hover:text-blue-700">
                        "View summary"
                    </a>
                </div>

                <Suspense fallback=move || view! {
                    <p class="text-sm text-gray-400 text-center py-8">"Loading squad…"</p>
                }>
                    {move || {
                        let squad = players.get().and_then(|result| result.ok())?;
                        let tariff = fine_types.get().and_then(|result| result.ok())?;
                        Some(view! {
                            <RecordFineForm
                                squad=squad.clone()
                                tariff=tariff
                                ledger_version=ledger_version
                            />
                            <RecordPaymentForm squad=squad ledger_version=ledger_version />
                        }.into_any())
                    }}
                </Suspense>

                <RecentEntries ledger_version=ledger_version />
            </div>
        </main>
    }
    .into_any()
}

#[component]
fn RecentEntries(ledger_version: RwSignal<u32>) -> impl IntoView {
    let entries = Resource::new(move || ledger_version.get(), |_| get_recent_entries());
    let delete_error = RwSignal::new(Option::<String>::None);

    let on_delete = move |entry_id: i32, is_payment: bool| {
        delete_error.set(None);
        leptos::task::spawn_local(async move {
            match delete_entry(entry_id, is_payment).await {
                Ok(()) => ledger_version.update(|version| *version = version.wrapping_add(1)),
                Err(error) => delete_error.set(Some(error.to_string())),
            }
        });
    };

    view! {
        <div class="bg-white rounded-xl shadow-md overflow-hidden">
            <div class="px-6 py-4 bg-gray-50 border-b border-gray-100">
                <h2 class="font-bold text-gray-800">"Recent entries"</h2>
            </div>
            <Show when=move || delete_error.get().is_some()>
                <p class="text-sm text-red-500 px-6 pt-3">
                    {move || delete_error.get().unwrap_or_default()}
                </p>
            </Show>
            <Transition fallback=move || view! {
                <p class="text-sm text-gray-400 text-center py-8">"Loading…"</p>
            }>
                {move || {
                    let recent = entries.get()?.ok()?;
                    if recent.is_empty() {
                        return Some(view! {
                            <p class="text-sm text-gray-400 text-center py-8">
                                "Nothing recorded yet."
                            </p>
                        }.into_any());
                    }
                    Some(view! {
                        <ul class="divide-y divide-gray-50">
                            <For
                                each=move || recent.clone()
                                key=|entry| (entry.is_payment, entry.entry_id)
                                children=move |entry| {
                                    let entry_id = entry.entry_id;
                                    let is_payment = entry.is_payment;
                                    let note = entry.note.clone().unwrap_or_default();
                                    view! {
                                        <li class="px-6 py-3 flex items-center justify-between gap-3">
                                            <div class="min-w-0">
                                                <p class="text-sm text-gray-800 truncate">
                                                    {entry.player_name}
                                                </p>
                                                <p class="text-xs text-gray-400 mt-0.5 truncate">
                                                    {entry.description}
                                                    " · "
                                                    {entry.happened_on}
                                                    {if note.is_empty() {
                                                        String::new()
                                                    } else {
                                                        format!(" · {note}")
                                                    }}
                                                </p>
                                            </div>
                                            <div class="flex items-center gap-3 shrink-0">
                                                <span class=if is_payment {
                                                    "font-mono text-sm text-green-600"
                                                } else {
                                                    "font-mono text-sm text-gray-800"
                                                }>
                                                    {format_pence(entry.amount_pence)}
                                                </span>
                                                <button
                                                    type="button"
                                                    class="text-xs text-gray-400 hover:text-red-600 cursor-pointer"
                                                    on:click=move |_| on_delete(entry_id, is_payment)
                                                >
                                                    "Undo"
                                                </button>
                                            </div>
                                        </li>
                                    }.into_any()
                                }
                            />
                        </ul>
                    }.into_any())
                }}
            </Transition>
        </div>
    }
    .into_any()
}

#[component]
fn RecordFineForm(
    squad: Vec<SquadPlayer>,
    tariff: Vec<FineType>,
    ledger_version: RwSignal<u32>,
) -> impl IntoView {
    let toaster = use_toaster();
    let selected_player = RwSignal::new(Option::<i32>::None);
    let selected_fine_type = RwSignal::new(Option::<i32>::None);
    let note = RwSignal::new(String::new());
    let is_saving = RwSignal::new(false);
    let reset_fields = RwSignal::new(0_u32);

    let player_options: Vec<SelectOption> = squad
        .into_iter()
        .map(|player| SelectOption {
            value: player.squad_player_id,
            label: player.name,
        })
        .collect();

    let fine_options: Vec<SelectOption> = tariff
        .into_iter()
        .map(|fine_type| SelectOption {
            value: fine_type.fine_type_id,
            label: format!(
                "{} — {}",
                fine_type.name,
                format_pence(fine_type.default_amount_pence),
            ),
        })
        .collect();

    let on_submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();

        let Some(squad_player_id) = selected_player.get() else {
            if let Some(toaster) = toaster {
                toaster.show_error("Pick a player.");
            }
            return;
        };
        let Some(fine_type_id) = selected_fine_type.get() else {
            if let Some(toaster) = toaster {
                toaster.show_error("Pick a fine.");
            }
            return;
        };

        let note_value = note.get();
        is_saving.set(true);
        leptos::task::spawn_local(async move {
            match record_fine(squad_player_id, fine_type_id, note_value).await {
                Ok(()) => {
                    if let Some(toaster) = toaster {
                        toaster.show("Fine recorded");
                    }
                    note.set(String::new());
                    selected_player.set(None);
                    selected_fine_type.set(None);
                    reset_fields.update(|generation| *generation = generation.wrapping_add(1));
                    ledger_version.update(|version| *version = version.wrapping_add(1));
                }
                Err(error) => {
                    if let Some(toaster) = toaster {
                        toaster.show_error(error.to_string());
                    }
                }
            }
            is_saving.set(false);
        });
    };

    view! {
        <form on:submit=on_submit class="bg-white rounded-xl shadow-md p-6 space-y-4">
            <h2 class="font-bold text-gray-800">"Record a fine"</h2>

            <SearchableSelect
                options=player_options
                placeholder="Search players…"
                selected=selected_player
                reset=reset_fields
            />

            <SearchableSelect
                options=fine_options
                placeholder="Search fines…"
                selected=selected_fine_type
                reset=reset_fields
            />

            <input
                type="text"
                placeholder="Note (optional)"
                prop:value=move || note.get()
                on:input=move |event| note.set(event_target_value(&event))
                class="w-full border border-gray-300 rounded-lg px-3 py-2"
            />

            <button
                type="submit"
                disabled=move || is_saving.get()
                class="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
            >
                {move || if is_saving.get() { "Saving…" } else { "Add fine" }}
            </button>
        </form>
    }
    .into_any()
}

#[component]
fn RecordPaymentForm(squad: Vec<SquadPlayer>, ledger_version: RwSignal<u32>) -> impl IntoView {
    let toaster = use_toaster();
    let selected_player = RwSignal::new(Option::<i32>::None);
    let amount_text = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let is_saving = RwSignal::new(false);
    let reset_fields = RwSignal::new(0_u32);

    let player_options: Vec<SelectOption> = squad
        .into_iter()
        .map(|player| SelectOption {
            value: player.squad_player_id,
            label: player.name,
        })
        .collect();

    let on_submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();

        let Some(squad_player_id) = selected_player.get() else {
            if let Some(toaster) = toaster {
                toaster.show_error("Pick a player.");
            }
            return;
        };
        let Some(amount_pence) = parse_pounds_to_pence(&amount_text.get()) else {
            if let Some(toaster) = toaster {
                toaster.show_error("Amount must look like 5 or 5.50.");
            }
            return;
        };
        if amount_pence == 0 {
            if let Some(toaster) = toaster {
                toaster.show_error("Amount must be more than zero.");
            }
            return;
        }

        let note_value = note.get();
        is_saving.set(true);
        leptos::task::spawn_local(async move {
            match record_payment(squad_player_id, amount_pence, note_value).await {
                Ok(()) => {
                    if let Some(toaster) = toaster {
                        toaster.show(format!("Payment of {} recorded", format_pence(amount_pence)));
                    }
                    amount_text.set(String::new());
                    reset_fields.update(|generation| *generation = generation.wrapping_add(1));
                    note.set(String::new());
                    selected_player.set(None);
                    ledger_version.update(|version| *version = version.wrapping_add(1));
                }
                Err(error) => {
                    if let Some(toaster) = toaster {
                        toaster.show_error(error.to_string());
                    }
                }
            }
            is_saving.set(false);
        });
    };

    view! {
        <form on:submit=on_submit class="bg-white rounded-xl shadow-md p-6 space-y-4">
            <h2 class="font-bold text-gray-800">"Record a payment"</h2>

            <SearchableSelect
                options=player_options
                placeholder="Search players…"
                selected=selected_player
                reset=reset_fields
            />

            <div>
                <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                    "Amount"
                </label>
                <input
                    type="text"
                    inputmode="decimal"
                    placeholder="5.00"
                    prop:value=move || amount_text.get()
                    on:input=move |event| amount_text.set(event_target_value(&event))
                    class="w-full border border-gray-300 rounded-lg px-3 py-2 mt-1 font-mono"
                />
            </div>

            <input
                type="text"
                placeholder="Note (optional)"
                prop:value=move || note.get()
                on:input=move |event| note.set(event_target_value(&event))
                class="w-full border border-gray-300 rounded-lg px-3 py-2"
            />

            <button
                type="submit"
                disabled=move || is_saving.get()
                class="w-full bg-green-600 hover:bg-green-700 text-white font-medium py-2 px-4 rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
            >
                {move || if is_saving.get() { "Saving…" } else { "Add payment" }}
            </button>
        </form>
    }
    .into_any()
}


#[cfg(test)]
mod tests {
    use super::parse_pounds_to_pence;

    #[test]
    fn parses_plain_and_decimal_amounts() {
        assert_eq!(parse_pounds_to_pence("5"), Some(500));
        assert_eq!(parse_pounds_to_pence("5.50"), Some(550));
        assert_eq!(parse_pounds_to_pence("0.50"), Some(50));
        assert_eq!(parse_pounds_to_pence("10.05"), Some(1005));
    }

    #[test]
    fn tolerates_currency_symbol_and_spacing() {
        assert_eq!(parse_pounds_to_pence(" £5.50 "), Some(550));
        assert_eq!(parse_pounds_to_pence("£10"), Some(1000));
    }

    #[test]
    fn treats_single_decimal_as_tenths() {
        assert_eq!(parse_pounds_to_pence("5.5"), Some(550));
    }

    #[test]
    fn rejects_nonsense() {
        assert_eq!(parse_pounds_to_pence("abc"), None);
        assert_eq!(parse_pounds_to_pence("5.555"), None);
        assert_eq!(parse_pounds_to_pence("-5"), None);
        assert_eq!(parse_pounds_to_pence(""), None);
        assert_eq!(parse_pounds_to_pence("."), None);
        assert_eq!(parse_pounds_to_pence("£"), None);
    }
}
