#![expect(
    clippy::mem_forget,
    reason = "the island macro forgets the render state it built on the server"
)]

use design_system::formatting::format_file_size;
use design_system::icons::{icon, LucideGlyph};
use design_system::{
    button_class, button_icon_class, Alert, ButtonSize, ButtonTone, EmptyState, Field,
    LoadingState, PageHeader, PageSection, Textarea, Tone,
};
use leptos::ev::DragEvent;
use leptos::prelude::*;

use crate::components::admin_only::AdminOnly;
use crate::server::source_pages::{list_fulltime_page_links, list_source_pages, SaveSourcePage};
use crate::types::{FulltimePageLink, SavedSourcePage, StoredSourcePage};

const PAGE_SOURCE_ROWS: u32 = 12;
const FETCHED_AT_FORMAT: &str = "%-d %b %Y, %H:%M";
const DETAIL_SEPARATOR: &str = " · ";

#[component]
pub fn SourcePagesAdminPage() -> impl IntoView {
    view! {
        <AdminOnly>
            <SourcePagesEditor />
        </AdminOnly>
    }
    .into_any()
}

#[island]
fn SourcePagesEditor() -> impl IntoView {
    let save_page = ServerAction::<SaveSourcePage>::new();
    let page_source = RwSignal::new(String::new());
    let drop_failure = RwSignal::new(None::<String>);
    let stored_pages = Resource::new(move || save_page.version().get(), |_| list_source_pages());
    let page_links = Resource::new(|| (), |()| list_fulltime_page_links());

    Effect::new(move |_| {
        if matches!(save_page.value().get(), Some(Ok(_))) {
            page_source.set(String::new());
        }
    });

    let has_no_source = move || page_source.with(|source| source.trim().is_empty());
    let save = move |_| {
        drop_failure.set(None);
        save_page.dispatch(SaveSourcePage {
            html: page_source.get_untracked(),
        });
    };

    view! {
        <main class="mx-auto max-w-3xl space-y-8 px-4 py-8">
            <PageHeader
                title="FA Full-Time Pages"
                description="Pages saved from FA Full-Time by hand. Saving one updates the York fixtures everyone sees."
            />
            <PageSection title="Add a Page">
                <div
                    class="space-y-4"
                    on:dragover=allow_file_drop
                    on:drop=move |event| load_dropped_file(&event, page_source, drop_failure)
                >
                    <Transition fallback=|| ()>
                        {move || Suspend::new(async move {
                            match page_links.await {
                                Ok(links) => view! { <FulltimePageLinks links=links /> }.into_any(),
                                Err(error) => {
                                    view! {
                                        <Alert
                                            tone=Tone::Critical
                                            title="Couldn't Load the Full-Time Links"
                                        >
                                            {server_error_message(error)}
                                        </Alert>
                                    }
                                        .into_any()
                                }
                            }
                        })}
                    </Transition>
                    <Field
                        label="Page Source"
                        description=|| {
                            "Open the results, fixtures or team page, view its source and paste all of it here, or drop a saved .html file here instead."
                        }
                    >
                        <Textarea
                            value=page_source
                            rows=PAGE_SOURCE_ROWS
                            placeholder="Paste Page Source or Drop an HTML File"
                            disabled=save_page.pending()
                        />
                    </Field>
                    {move || {
                        drop_failure
                            .get()
                            .map(|reason| {
                                view! {
                                    <Alert tone=Tone::Critical title="Couldn't Read the File">
                                        {reason}
                                    </Alert>
                                }
                            })
                    }}
                    <SaveOutcome outcome=save_page.value() />
                    <button
                        type="button"
                        class=button_class(ButtonTone::Primary, ButtonSize::Standard)
                        disabled=move || has_no_source() || save_page.pending().get()
                        on:click=save
                    >
                        {move || if save_page.pending().get() { "Saving…" } else { "Save Page" }}
                    </button>
                </div>
            </PageSection>
            <PageSection title="Stored Pages">
                <Transition fallback=|| {
                    view! { <LoadingState /> }
                }>
                    {move || Suspend::new(async move {
                        match stored_pages.await {
                            Ok(pages) => view! { <StoredPageList pages=pages /> }.into_any(),
                            Err(error) => {
                                view! {
                                    <Alert
                                        tone=Tone::Critical
                                        title="Couldn't Load the Stored Pages"
                                    >
                                        {server_error_message(error)}
                                    </Alert>
                                }
                                    .into_any()
                            }
                        }
                    })}
                </Transition>
            </PageSection>
        </main>
    }
    .into_any()
}

fn allow_file_drop(event: DragEvent) {
    event.prevent_default();
}

#[cfg(feature = "hydrate")]
fn load_dropped_file(
    event: &DragEvent,
    page_source: RwSignal<String>,
    drop_failure: RwSignal<Option<String>>,
) {
    let Some(file) = event
        .data_transfer()
        .and_then(|transfer| transfer.files())
        .and_then(|files| files.get(0))
    else {
        return;
    };
    event.prevent_default();
    drop_failure.set(None);
    leptos::task::spawn_local(async move {
        let text = wasm_bindgen_futures::JsFuture::from(file.text()).await;
        match text.ok().and_then(|value| value.as_string()) {
            Some(html) => page_source.set(html),
            None => drop_failure.set(Some(format!("{} could not be read as text.", file.name()))),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_dropped_file(
    _event: &DragEvent,
    _page_source: RwSignal<String>,
    _drop_failure: RwSignal<Option<String>>,
) {
}

fn server_error_message(error: ServerFnError) -> String {
    match error {
        ServerFnError::ServerError(message) => message,
        other => other.to_string(),
    }
}

fn page_kind_label(page_kind: &str) -> String {
    let mut characters = page_kind.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().chain(characters).collect())
        .unwrap_or_default()
}

fn publication_summary(saved: &SavedSourcePage) -> String {
    let published = format!(
        "The site now shows {} York fixtures.",
        saved.fixtures_published
    );
    if saved.fixtures_removed == 0 {
        return published;
    }
    format!(
        "{published} {} upcoming fixtures no longer listed on Full-Time were marked removed.",
        saved.fixtures_removed
    )
}

fn saved_summary(saved: &SavedSourcePage) -> String {
    let mut counts = Vec::new();
    if saved.fixtures_parsed > 0 {
        counts.push(format!("{} fixtures", saved.fixtures_parsed));
    }
    if saved.players_parsed > 0 {
        counts.push(format!("{} players", saved.players_parsed));
    }
    if counts.is_empty() {
        return "Nothing was listed on it.".to_owned();
    }
    format!("Read {}.", counts.join(" and "))
}

#[component]
fn FulltimePageLinks(links: Vec<FulltimePageLink>) -> impl IntoView {
    let link_size = ButtonSize::Compact;
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <span class="text-sm text-gray-600">"Open on FA Full-Time:"</span>
            {links
                .into_iter()
                .map(|link| {
                    view! {
                        <a
                            href=link.url
                            target="_blank"
                            rel="noopener noreferrer"
                            class=button_class(ButtonTone::Secondary, link_size)
                        >
                            {page_kind_label(&link.page_kind)}
                            {icon(LucideGlyph::ExternalLink, button_icon_class(link_size))}
                        </a>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

#[component]
fn SaveOutcome(
    #[prop(into)] outcome: Signal<Option<Result<SavedSourcePage, ServerFnError>>>,
) -> impl IntoView {
    move || {
        outcome.get().map(|result| match result {
            Ok(saved) => view! { <SavedPageAlert saved=saved /> }.into_any(),
            Err(error) => view! {
                <Alert tone=Tone::Critical title="Couldn't Save the Page">
                    {server_error_message(error)}
                </Alert>
            }
            .into_any(),
        })
    }
}

#[component]
fn SavedPageAlert(saved: SavedSourcePage) -> impl IntoView {
    let title = format!("Saved the {} Page", page_kind_label(&saved.page_kind));
    let summary = saved_summary(&saved);
    let publication = publication_summary(&saved);
    let tone = if saved.issues.is_empty() {
        Tone::Positive
    } else {
        Tone::Caution
    };
    let issues = (!saved.issues.is_empty()).then(|| {
        view! {
            <p class="mt-2">"These values were not recognised:"</p>
            <ul class="mt-1 list-disc space-y-1 pl-5">
                {saved.issues.into_iter().map(|issue| view! { <li>{issue}</li> }).collect_view()}
            </ul>
        }
    });
    view! {
        <Alert tone=tone title=title>
            <p>{summary}</p>
            <p>{publication}</p>
            {issues}
        </Alert>
    }
    .into_any()
}

#[component]
fn StoredPageList(pages: Vec<StoredSourcePage>) -> impl IntoView {
    if pages.is_empty() {
        return view! {
            <EmptyState
                title="No Pages Yet"
                description="Pages added here, and pages the scrape fetches, appear here."
            />
        }
        .into_any();
    }
    view! {
        <ul class="divide-y divide-gray-100 rounded-lg border border-gray-200 bg-white">
            {pages.into_iter().map(|page| view! { <StoredPageRow page=page /> }).collect_view()}
        </ul>
    }
    .into_any()
}

#[component]
fn StoredPageRow(page: StoredSourcePage) -> impl IntoView {
    let size = format_file_size(u64::try_from(page.size_bytes).unwrap_or_default());
    let fetched_at = page.fetched_at.format(FETCHED_AT_FORMAT).to_string();
    let details = [page.page_key, size, fetched_at].join(DETAIL_SEPARATOR);
    view! {
        <li class="flex flex-col gap-0.5 px-4 py-3">
            <span class="text-sm font-medium text-gray-900">
                {page_kind_label(&page.page_kind)}
            </span>
            <span class="text-xs text-gray-500">{details}</span>
        </li>
    }
}
