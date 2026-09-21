use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

impl StatusMessage {
    pub fn confirmation(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
        }
    }

    pub fn failure(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
        }
    }
}

#[component]
pub fn StatusLine(status: RwSignal<Option<StatusMessage>>) -> impl IntoView {
    view! {
        {move || {
            let current = status.get()?;
            Some(
                view! {
                    <p class=if current.is_error {
                        "text-sm text-red-600"
                    } else {
                        "text-sm text-green-700"
                    }>{current.text}</p>
                },
            )
        }}
    }
    .into_any()
}
