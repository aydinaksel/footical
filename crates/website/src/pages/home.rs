use crate::components::team_picker::TeamPicker;
use crate::server::data::get_team_listing;
use crate::tracked_team::{use_tracked_team, TrackedTeam};
use crate::types::TeamListing;
use crate::CALENDAR_HOST;
use leptos::prelude::*;
use leptos_use::{use_clipboard, UseClipboardReturn};

#[component]
pub fn Home() -> impl IntoView {
    let TrackedTeam {
        team_id: tracked_team_id,
        ..
    } = use_tracked_team();

    let tracked_team = Resource::new_blocking(
        move || tracked_team_id.get(),
        |team_id| async move {
            match team_id {
                Some(team_id) => get_team_listing(team_id).await,
                None => Ok(None),
            }
        },
    );

    view! {
        <main class="flex justify-center p-4 pt-8">
            <div class="w-full max-w-md">
                <Suspense fallback=|| ()>
                    {move || Suspend::new(async move {
                        match tracked_team.await {
                            Ok(Some(team)) => view! { <SubscriptionDetails team=team /> }.into_any(),
                            Ok(None) => view! { <SubscribePrompt /> }.into_any(),
                            Err(_) => view! { <SubscriptionUnavailable /> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </main>
    }
    .into_any()
}

#[component]
fn SubscribePrompt() -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl shadow-md p-8 space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-gray-800">"Subscribe"</h1>
                <p class="text-sm text-gray-500 mt-1">
                    "Search for your team to get subscription instructions."
                </p>
            </div>
            <TeamPicker />
        </div>
    }
    .into_any()
}

#[component]
fn SubscriptionUnavailable() -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl shadow-md p-8">
            <h1 class="text-2xl font-bold text-gray-800">"Subscribe"</h1>
            <p class="text-sm text-gray-500 mt-1">
                "Your team could not be loaded. Reload the page to try again."
            </p>
        </div>
    }
    .into_any()
}

#[component]
fn SubscriptionDetails(team: TeamListing) -> impl IntoView {
    let TrackedTeam { set_team_id, .. } = use_tracked_team();
    let UseClipboardReturn {
        copied: is_copied,
        copy: copy_to_clipboard,
        ..
    } = use_clipboard();
    let copy_to_clipboard = StoredValue::new(copy_to_clipboard);

    let calendar_url = format!("https://{CALENDAR_HOST}/{}.ics", team.team_id);
    let webcal_url = format!("webcal://{CALENDAR_HOST}/{}.ics", team.team_id);
    let google_calendar_url = format!(
        "https://calendar.google.com/calendar/r?cid=webcal%3A%2F%2F{CALENDAR_HOST}%2F{}.ics",
        team.team_id,
    );
    let copyable_url = calendar_url.clone();

    view! {
        <div class="bg-white rounded-xl shadow-md overflow-hidden">
            <div class="px-6 py-5 border-b border-gray-100 flex items-start justify-between">
                <div>
                    <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                        "Subscribe"
                    </p>
                    <h1 class="text-xl font-bold text-gray-800 mt-0.5">{team.team_name}</h1>
                </div>
                <button
                    class="text-sm text-gray-400 hover:text-gray-600 transition-colors mt-0.5 cursor-pointer"
                    on:click=move |_| set_team_id.set(None)
                >
                    "Change team"
                </button>
            </div>

            <div class="px-6 py-5 border-b border-gray-100">
                <p class="text-xs font-medium text-gray-400 uppercase tracking-wider mb-2">
                    "Your calendar link"
                </p>
                <div class="flex items-center gap-2">
                    <span class="flex-1 text-sm text-gray-600 font-mono bg-gray-50 px-3 py-2 rounded-lg truncate">
                        {calendar_url}
                    </span>
                    <button
                        class="shrink-0 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-2 rounded-lg transition-colors cursor-pointer"
                        on:click=move |_| {
                            copy_to_clipboard.with_value(|copy| copy(&copyable_url));
                        }
                    >
                        {move || if is_copied.get() { "Copied!" } else { "Copy" }}
                    </button>
                </div>
            </div>

            <div class="px-6 py-5 border-b border-gray-100 flex items-center justify-between gap-4">
                <div>
                    <p class="font-medium text-gray-800">"Apple Calendar"</p>
                    <p class="text-sm text-gray-400 mt-0.5">
                        "Opens directly on iPhone, iPad & Mac"
                    </p>
                </div>
                <a
                    href=webcal_url
                    class="shrink-0 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-2 rounded-lg transition-colors cursor-pointer"
                >
                    "Subscribe"
                </a>
            </div>

            <div class="px-6 py-5 border-b border-gray-100 flex items-center justify-between gap-4">
                <div>
                    <p class="font-medium text-gray-800">"Google Calendar"</p>
                    <p class="text-sm text-gray-400 mt-0.5">"Opens in a new tab"</p>
                </div>
                <a
                    href=google_calendar_url
                    target="_blank"
                    rel="noopener noreferrer"
                    class="shrink-0 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-2 rounded-lg transition-colors cursor-pointer"
                >
                    "Subscribe"
                </a>
            </div>

            <div class="px-6 py-5">
                <p class="font-medium text-gray-800 mb-2">"Outlook"</p>
                <p class="text-sm text-gray-500 mb-3">
                    "Copy the link above, then follow the steps for your version:"
                </p>
                <div class="space-y-3">
                    <div>
                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">
                            "Web (Outlook.com or Office 365)"
                        </p>
                        <p class="text-sm text-gray-600">
                            "Settings → View all Outlook settings → Calendar → Shared calendars → Subscribe from web → paste the link"
                        </p>
                    </div>
                    <div>
                        <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">
                            "Desktop app"
                        </p>
                        <p class="text-sm text-gray-600">
                            "File → Account Settings → Internet Calendars → New → paste the link"
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
    .into_any()
}
