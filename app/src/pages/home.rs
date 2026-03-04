use crate::clipboard::copy_to_clipboard;
use crate::components::team_picker::TeamPicker;
use crate::types::{Team, clear_tracked_team_id};
use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn Home() -> impl IntoView {
    let all_teams = use_context::<RwSignal<Vec<Team>>>().expect("all_teams context");
    let tracked_team_id = use_context::<RwSignal<Option<i32>>>().expect("tracked_team_id context");
    let is_copied = RwSignal::new(false);

    let tracked_team = Memo::new(move |_| -> Option<Team> {
        tracked_team_id.get().and_then(|team_id| {
            all_teams
                .get()
                .into_iter()
                .find(|team| team.team_id == team_id)
        })
    });

    let calendar_url = Memo::new(move |_| -> Option<String> {
        tracked_team_id
            .get()
            .map(|team_id| format!("https://calendar.footical.club/{}.ics", team_id))
    });

    let on_change_team = move |_: web_sys::MouseEvent| {
        clear_tracked_team_id();
        tracked_team_id.set(None);
    };

    let on_copy_click = move |_: web_sys::MouseEvent| {
        if let Some(url) = calendar_url.get() {
            spawn_local(async move {
                if copy_to_clipboard(&url).await {
                    is_copied.set(true);
                    let reset_timeout = Timeout::new(1500, move || {
                        is_copied.set(false);
                    });
                    reset_timeout.forget();
                }
            });
        }
    };

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="bg-white rounded-xl shadow-md p-8 w-full max-w-md">
                <Show
                    when=move || tracked_team.get().is_some()
                    fallback=|| view! {
                        <div class="space-y-4">
                            <div>
                                <h1 class="text-2xl font-bold text-gray-800">"iCal Link Generator"</h1>
                                <p class="text-sm text-gray-500 mt-1">
                                    "Search for your team to get a calendar link."
                                </p>
                            </div>
                            <TeamPicker />
                        </div>
                    }
                >
                    <div class="space-y-4">
                        <div class="flex items-start justify-between">
                            <div>
                                <h1 class="text-2xl font-bold text-gray-800">"iCal Link Generator"</h1>
                                <p class="text-lg text-gray-600 mt-1">
                                    {move || {
                                        tracked_team.get().map(|team| team.name).unwrap_or_default()
                                    }}
                                </p>
                            </div>
                            <button
                                class="text-sm text-gray-400 hover:text-gray-600 transition-colors mt-1 cursor-pointer shrink-0"
                                on:click=on_change_team
                            >
                                "Change team"
                            </button>
                        </div>
                        <button
                            class="w-full bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-semibold py-2 px-4 rounded-lg transition-colors duration-150 cursor-pointer"
                            on:click=on_copy_click
                        >
                            {move || if is_copied.get() { "Copied!" } else { "Copy Calendar Link" }}
                        </button>
                    </div>
                </Show>
            </div>
        </main>
    }
}
