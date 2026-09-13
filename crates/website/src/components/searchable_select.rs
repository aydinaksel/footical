use leptos::prelude::*;
use leptos_use::on_click_outside;

const MAXIMUM_VISIBLE_OPTIONS: usize = 50;

#[derive(Clone, PartialEq)]
pub struct SelectOption {
    pub value: i32,
    pub label: String,
}

#[component]
pub fn SearchableSelect(
    options: Vec<SelectOption>,
    placeholder: &'static str,
    selected: RwSignal<Option<i32>>,
    reset: RwSignal<u32>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let is_open = RwSignal::new(false);
    let all_options = StoredValue::new(options);

    let matching = Memo::new(move |_| -> Vec<SelectOption> {
        let search_text = query.get().to_lowercase();
        all_options.with_value(|options: &Vec<SelectOption>| {
            options
                .iter()
                .filter(|option| {
                    search_text.is_empty() || option.label.to_lowercase().contains(&search_text)
                })
                .take(MAXIMUM_VISIBLE_OPTIONS)
                .cloned()
                .collect()
        })
    });

    let container = NodeRef::<leptos::html::Div>::new();
    let _stop_click_outside = on_click_outside(container, move |_| is_open.set(false));

    Effect::new(move |previous_generation: Option<u32>| {
        let generation = reset.get();
        if previous_generation.is_some_and(|previous| previous != generation) {
            query.set(String::new());
            is_open.set(false);
        }
        generation
    });

    view! {
        <div class="relative" node_ref=container>
            <input
                type="text"
                placeholder=placeholder
                prop:value=move || query.get()
                on:input=move |event| {
                    query.set(event_target_value(&event));
                    selected.set(None);
                    is_open.set(true);
                }
                on:focus=move |_| is_open.set(true)
                class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            {move || {
                if !is_open.get() || matching.get().is_empty() {
                    return None;
                }
                Some(view! {
                    <ul class="absolute z-20 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-60 overflow-y-auto">
                        <For
                            each=move || matching.get()
                            key=|option| option.value
                            children=move |option| {
                                let value = option.value;
                                let label = option.label.clone();
                                view! {
                                    <li>
                                        <button
                                            type="button"
                                            class="w-full text-left px-4 py-2.5 hover:bg-blue-50 cursor-pointer border-b border-gray-50 last:border-0 text-sm text-gray-800"
                                            on:click=move |_| {
                                                selected.set(Some(value));
                                                query.set(label.clone());
                                                is_open.set(false);
                                            }
                                        >
                                            {option.label}
                                        </button>
                                    </li>
                                }
                                .into_any()
                            }
                        />
                    </ul>
                })
            }}
        </div>
    }
    .into_any()
}
