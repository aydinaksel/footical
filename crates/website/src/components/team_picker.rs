use crate::tracked_team::{TrackedTeam, use_tracked_team};
use crate::types::{Division, League, Team};
use leptos::prelude::*;
use leptos_use::on_click_outside;

#[derive(Clone, PartialEq)]
struct TeamOption {
    team_id: i32,
    team_name: String,
    division_name: String,
    league_name: String,
}

#[component]
pub fn TeamPicker() -> impl IntoView {
    let all_leagues = use_context::<RwSignal<Vec<League>>>().unwrap_or_default();
    let all_divisions = use_context::<RwSignal<Vec<Division>>>().unwrap_or_default();
    let all_teams = use_context::<RwSignal<Vec<Team>>>().unwrap_or_default();
    let TrackedTeam { set_team_id, .. } = use_tracked_team();

    let query = RwSignal::new(String::new());
    let is_open = RwSignal::new(false);

    let filtered_options = Memo::new(move |_| -> Vec<TeamOption> {
        let search_text = query.get().to_lowercase();
        if search_text.is_empty() {
            return vec![];
        }
        let divisions = all_divisions.get();
        let leagues = all_leagues.get();
        all_teams
            .get()
            .into_iter()
            .filter_map(|team| {
                let division = divisions
                    .iter()
                    .find(|division| division.division_id == team.division_id)?;
                let league = leagues
                    .iter()
                    .find(|league| league.league_id == division.league_id)?;
                if team.name.to_lowercase().contains(&search_text) {
                    Some(TeamOption {
                        team_id: team.team_id,
                        team_name: team.name.clone(),
                        division_name: division.name.clone(),
                        league_name: league.name.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    });

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
            {move || {
                if !is_open.get() || filtered_options.get().is_empty() {
                    return None;
                }
                Some(view! {
                    <ul class="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-60 overflow-y-auto">
                        <For
                            each=move || filtered_options.get()
                            key=|option| option.team_id
                            children=move |option| {
                                let team_id = option.team_id;
                                let display_name = option.team_name.clone();
                                let context_label = format!(
                                    "{}, {}",
                                    option.division_name, option.league_name
                                );
                                let team_name = option.team_name;
                                view! {
                                    <li>
                                        <button
                                            type="button"
                                            class="w-full text-left px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-50 last:border-0"
                                            on:click=move |_| {
                                                set_team_id.set(Some(team_id));
                                                query.set(team_name.clone());
                                                is_open.set(false);
                                            }
                                        >
                                            <span class="font-medium text-gray-800">{display_name}</span>
                                            <span class="text-sm text-gray-400 ml-2">{context_label}</span>
                                        </button>
                                    </li>
                                }
                            }
                        />
                    </ul>
                })
            }}
        </div>
    }
    .into_any()
}
