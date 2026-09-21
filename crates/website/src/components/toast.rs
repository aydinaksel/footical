use leptos::prelude::*;
use leptos_toaster::{Toast, ToastId, ToastVariant, Toasts};

pub fn show_toast(message: impl Into<String>, variant: ToastVariant) {
    let Some(toaster) = use_context::<Toasts>() else {
        return;
    };

    let message = message.into();
    let toast_id = ToastId::new();
    toaster.toast(
        move || {
            let message = message.clone();
            view! { <Toast toast_id variant=variant title=move || message.clone() /> }
        },
        Some(toast_id),
        None,
    );
}
