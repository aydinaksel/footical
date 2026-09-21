#![expect(
    clippy::mem_forget,
    reason = "the island macro forgets the render state it built on the server"
)]

use crate::components::browser_navigation::reload_page;
use crate::tracked_team::use_tracked_team;
use leptos::prelude::*;
use leptos_use::{use_clipboard, UseClipboardReturn};

#[island]
pub fn ChangeTeamButton() -> impl IntoView {
    let set_team_id = use_tracked_team().set_team_id;

    view! {
        <button
            class="text-sm text-gray-400 hover:text-gray-600 transition-colors mt-0.5 cursor-pointer"
            on:click=move |_| {
                set_team_id.set(None);
                reload_page();
            }
        >
            "Change team"
        </button>
    }
    .into_any()
}

#[island]
pub fn CopyCalendarLinkButton(calendar_url: String) -> impl IntoView {
    let UseClipboardReturn {
        copied: is_copied,
        copy: copy_to_clipboard,
        ..
    } = use_clipboard();
    let copy_to_clipboard = StoredValue::new(copy_to_clipboard);
    let calendar_url = StoredValue::new(calendar_url);

    view! {
        <button
            class="shrink-0 text-sm font-medium text-blue-600 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-3 py-2 rounded-lg transition-colors cursor-pointer"
            on:click=move |_| {
                calendar_url.with_value(|url| copy_to_clipboard.with_value(|copy| copy(url)));
            }
        >
            {move || if is_copied.get() { "Copied!" } else { "Copy" }}
        </button>
    }
    .into_any()
}
