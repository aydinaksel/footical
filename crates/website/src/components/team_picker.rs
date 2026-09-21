#![expect(
    clippy::mem_forget,
    reason = "the island macro forgets the render state it built on the server"
)]

use crate::components::browser_navigation::reload_page;
use crate::server::data::search_teams;
use crate::tracked_team::use_tracked_team;
use crate::types::TeamListing;
use leptos::prelude::*;
use leptos_use::{on_click_outside, signal_debounced};

const SEARCH_DEBOUNCE_MILLISECONDS: f64 = 200.0;

#[island]
pub fn TeamPicker() -> impl IntoView {
    let set_team_id = use_tracked_team().set_team_id;

    let query = RwSignal::new(String::new());
    let is_open = RwSignal::new(false);
    let debounced_query = signal_debounced(query, SEARCH_DEBOUNCE_MILLISECONDS);

    let matching_teams = Resource::new(
        move || debounced_query.get(),
        |search_text| async move { search_teams(search_text).await },
    );

    let container = NodeRef::<leptos::html::Div>::new();
    let _stop_click_outside = on_click_outside(container, move |_| is_open.set(false));

    view! {
        <div class="relative" node_ref=container>
            <input
                type="text"
                placeholder="Search for your team…"
                prop:value=move || query.get()
                on:input=move |ev| {
                    query.set(event_target_value(&ev));
                    is_open.set(true);
                }
                on:focus=move |_| {
                    if !query.get().is_empty() {
                        is_open.set(true);
                    }
                }
                class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <Transition fallback=|| ()>
                {move || {
                    if !is_open.get() {
                        return None;
                    }
                    let teams = matching_teams.get()?.unwrap_or_default();
                    if teams.is_empty() {
                        return None;
                    }
                    Some(
                        view! {
                            <ul class="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-60 overflow-y-auto">
                                <For
                                    each=move || teams.clone()
                                    key=|team| team.team_id
                                    children=move |team| {
                                        view! {
                                            <TeamOption
                                                team=team
                                                on_pick=move |picked: TeamListing| {
                                                    set_team_id.set(Some(picked.team_id));
                                                    reload_page();
                                                }
                                            />
                                        }
                                    }
                                />
                            </ul>
                        },
                    )
                }}
            </Transition>
        </div>
    }
    .into_any()
}

#[component]
fn TeamOption(team: TeamListing, on_pick: impl Fn(TeamListing) + 'static) -> impl IntoView {
    let picked_team = team.clone();
    let context_label = format!("{}, {}", team.division_name, team.league_name);

    view! {
        <li>
            <button
                type="button"
                class="w-full text-left px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-50 last:border-0"
                on:click=move |_| on_pick(picked_team.clone())
            >
                <span class="font-medium text-gray-800">{team.team_name}</span>
                <span class="text-sm text-gray-400 ml-2">{context_label}</span>
            </button>
        </li>
    }
    .into_any()
}
