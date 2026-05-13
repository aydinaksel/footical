use crate::types::{Division, League, Team, save_tracked_team_id};
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
struct TeamOption {
    team_id: i32,
    team_name: String,
    division_name: String,
    league_name: String,
}

#[component]
pub fn TeamPicker() -> impl IntoView {
    let all_leagues = use_context::<RwSignal<Vec<League>>>().expect("all_leagues context");
    let all_divisions = use_context::<RwSignal<Vec<Division>>>().expect("all_divisions context");
    let all_teams = use_context::<RwSignal<Vec<Team>>>().expect("all_teams context");
    let tracked_team_id = use_context::<RwSignal<Option<i32>>>().expect("tracked_team_id context");

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

    view! {
        <div class="relative">
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
                on:blur=move |_| is_open.set(false)
                class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <Show when=move || is_open.get() && !filtered_options.get().is_empty()>
                <ul class="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-60 overflow-y-auto">
                    <For
                        each=move || filtered_options.get()
                        key=|option| option.team_id
                        children=move |option| {
                            let team_id = option.team_id;
                            let display_name = option.team_name.clone();
                            let context_label = format!(
                                "{} — {}",
                                option.division_name, option.league_name
                            );
                            let team_name = option.team_name;
                            view! {
                                <li>
                                    <button
                                        type="button"
                                        tabindex="-1"
                                        class="w-full text-left px-4 py-3 hover:bg-blue-50 cursor-pointer border-b border-gray-50 last:border-0"
                                        on:mousedown=|ev| ev.prevent_default()
                                        on:click=move |_| {
                                            save_tracked_team_id(team_id);
                                            tracked_team_id.set(Some(team_id));
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
            </Show>
        </div>
    }
}
