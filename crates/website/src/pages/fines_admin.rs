use crate::server::auth::check_auth;
use crate::server::squad::{get_fine_types, get_squad_players, record_fine, record_payment};
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
            Some(_) => {
                let navigate = leptos_router::hooks::use_navigate();
                navigate("/admin/login", Default::default());
                view! { <p class="text-sm text-gray-400 text-center py-16">"Redirecting…"</p> }
                    .into_any()
            }
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
                    <a href="/team" class="text-sm text-blue-600 hover:text-blue-700">
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
                            <RecordFineForm squad=squad.clone() tariff=tariff />
                            <RecordPaymentForm squad=squad />
                        }.into_any())
                    }}
                </Suspense>
            </div>
        </main>
    }
    .into_any()
}

#[component]
fn RecordFineForm(squad: Vec<SquadPlayer>, tariff: Vec<FineType>) -> impl IntoView {
    let selected_player = RwSignal::new(String::new());
    let selected_fine_type = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let feedback = RwSignal::new(Option::<String>::None);
    let is_error = RwSignal::new(false);
    let is_saving = RwSignal::new(false);

    let on_submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        feedback.set(None);

        let Ok(squad_player_id) = selected_player.get().parse::<i32>() else {
            is_error.set(true);
            feedback.set(Some("Pick a player.".to_owned()));
            return;
        };
        let Ok(fine_type_id) = selected_fine_type.get().parse::<i32>() else {
            is_error.set(true);
            feedback.set(Some("Pick a fine.".to_owned()));
            return;
        };

        let note_value = note.get();
        is_saving.set(true);
        leptos::task::spawn_local(async move {
            match record_fine(squad_player_id, fine_type_id, note_value).await {
                Ok(()) => {
                    is_error.set(false);
                    feedback.set(Some("Fine recorded.".to_owned()));
                    note.set(String::new());
                }
                Err(error) => {
                    is_error.set(true);
                    feedback.set(Some(error.to_string()));
                }
            }
            is_saving.set(false);
        });
    };

    view! {
        <form on:submit=on_submit class="bg-white rounded-xl shadow-md p-6 space-y-4">
            <h2 class="font-bold text-gray-800">"Record a fine"</h2>

            <select
                class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white"
                on:change=move |event| selected_player.set(event_target_value(&event))
            >
                <option value="">"Select player…"</option>
                {squad.into_iter().map(|player| view! {
                    <option value=player.squad_player_id.to_string()>{player.name}</option>
                }.into_any()).collect_view()}
            </select>

            <select
                class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white"
                on:change=move |event| selected_fine_type.set(event_target_value(&event))
            >
                <option value="">"Select fine…"</option>
                {tariff.into_iter().map(|fine_type| {
                    let label = format!(
                        "{} — {}",
                        fine_type.name,
                        format_pence(fine_type.default_amount_pence),
                    );
                    view! {
                        <option value=fine_type.fine_type_id.to_string()>{label}</option>
                    }.into_any()
                }).collect_view()}
            </select>

            <input
                type="text"
                placeholder="Note (optional)"
                prop:value=move || note.get()
                on:input=move |event| note.set(event_target_value(&event))
                class="w-full border border-gray-300 rounded-lg px-3 py-2"
            />

            <Feedback feedback=feedback is_error=is_error />

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
fn RecordPaymentForm(squad: Vec<SquadPlayer>) -> impl IntoView {
    let selected_player = RwSignal::new(String::new());
    let amount_text = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let feedback = RwSignal::new(Option::<String>::None);
    let is_error = RwSignal::new(false);
    let is_saving = RwSignal::new(false);

    let on_submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        feedback.set(None);

        let Ok(squad_player_id) = selected_player.get().parse::<i32>() else {
            is_error.set(true);
            feedback.set(Some("Pick a player.".to_owned()));
            return;
        };
        let Some(amount_pence) = parse_pounds_to_pence(&amount_text.get()) else {
            is_error.set(true);
            feedback.set(Some("Amount must look like 5 or 5.50.".to_owned()));
            return;
        };
        if amount_pence == 0 {
            is_error.set(true);
            feedback.set(Some("Amount must be more than zero.".to_owned()));
            return;
        }

        let note_value = note.get();
        is_saving.set(true);
        leptos::task::spawn_local(async move {
            match record_payment(squad_player_id, amount_pence, note_value).await {
                Ok(()) => {
                    is_error.set(false);
                    feedback.set(Some(format!("Recorded {}.", format_pence(amount_pence))));
                    amount_text.set(String::new());
                    note.set(String::new());
                }
                Err(error) => {
                    is_error.set(true);
                    feedback.set(Some(error.to_string()));
                }
            }
            is_saving.set(false);
        });
    };

    view! {
        <form on:submit=on_submit class="bg-white rounded-xl shadow-md p-6 space-y-4">
            <h2 class="font-bold text-gray-800">"Record a payment"</h2>

            <select
                class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white"
                on:change=move |event| selected_player.set(event_target_value(&event))
            >
                <option value="">"Select player…"</option>
                {squad.into_iter().map(|player| view! {
                    <option value=player.squad_player_id.to_string()>{player.name}</option>
                }.into_any()).collect_view()}
            </select>

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

            <Feedback feedback=feedback is_error=is_error />

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

#[component]
fn Feedback(feedback: RwSignal<Option<String>>, is_error: RwSignal<bool>) -> impl IntoView {
    view! {
        <Show when=move || feedback.get().is_some()>
            <p class=move || if is_error.get() {
                "text-sm text-red-500"
            } else {
                "text-sm text-green-600"
            }>
                {move || feedback.get().unwrap_or_default()}
            </p>
        </Show>
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
