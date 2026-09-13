use leptos::prelude::*;
use leptos_use::use_timeout_fn;

const TOAST_VISIBLE_MILLISECONDS: f64 = 2500.0;

#[derive(Clone, PartialEq)]
pub struct ToastMessage {
    pub text: String,
    pub is_error: bool,
}

#[derive(Clone, Copy)]
pub struct Toaster {
    message: RwSignal<Option<ToastMessage>>,
}

impl Toaster {
    pub fn show(&self, text: impl Into<String>) {
        self.message.set(Some(ToastMessage {
            text: text.into(),
            is_error: false,
        }));
    }

    pub fn show_error(&self, text: impl Into<String>) {
        self.message.set(Some(ToastMessage {
            text: text.into(),
            is_error: true,
        }));
    }
}

#[component]
pub fn ToastHost(children: ChildrenFn) -> impl IntoView {
    let message = RwSignal::new(Option::<ToastMessage>::None);
    provide_context(Toaster { message });

    let hide_after_delay = use_timeout_fn(
        move |_: ()| message.set(None),
        TOAST_VISIBLE_MILLISECONDS,
    );
    let start_hiding = hide_after_delay.start;

    Effect::new(move |_| {
        if message.get().is_some() {
            start_hiding(());
        }
    });

    view! {
        {children()}
        {move || {
            let current = message.get()?;
            Some(view! {
                <div
                    role="status"
                    aria-live="polite"
                    class="fixed inset-x-0 bottom-6 flex justify-center px-4 pointer-events-none z-50"
                >
                    <div class=if current.is_error {
                        "rounded-lg shadow-lg px-4 py-2.5 text-sm font-medium bg-red-600 text-white"
                    } else {
                        "rounded-lg shadow-lg px-4 py-2.5 text-sm font-medium bg-gray-900 text-white"
                    }>
                        {current.text}
                    </div>
                </div>
            }
            .into_any())
        }}
    }
    .into_any()
}

pub fn use_toaster() -> Option<Toaster> {
    use_context::<Toaster>()
}
