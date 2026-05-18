use crate::server::auth::login;
use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let password = RwSignal::new(String::new());
    let error_message = RwSignal::new(Option::<String>::None);
    let is_submitting = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let password_value = password.get();
        is_submitting.set(true);
        error_message.set(None);

        leptos::task::spawn_local(async move {
            match login(password_value).await {
                Ok(()) => {
                    let navigate = leptos_router::hooks::use_navigate();
                    navigate("/admin", Default::default());
                }
                Err(error) => {
                    error_message.set(Some(error.to_string()));
                    is_submitting.set(false);
                }
            }
        });
    };

    view! {
        <main class="flex justify-center p-4 pt-16">
            <div class="w-full max-w-sm">
                <div class="bg-white rounded-xl shadow-md p-8">
                    <h1 class="text-2xl font-bold text-gray-800 mb-6">"Admin Login"</h1>
                    <form on:submit=on_submit>
                        <div class="mb-4">
                            <input
                                type="password"
                                placeholder="Password"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                                class="w-full border border-gray-300 rounded-lg px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>
                        <Show when=move || error_message.get().is_some()>
                            <p class="text-sm text-red-500 mb-4">
                                {move || error_message.get().unwrap_or_default()}
                            </p>
                        </Show>
                        <button
                            type="submit"
                            disabled=move || is_submitting.get()
                            class="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors disabled:opacity-50 cursor-pointer"
                        >
                            {move || if is_submitting.get() { "Logging in…" } else { "Login" }}
                        </button>
                    </form>
                </div>
            </div>
        </main>
    }
    .into_any()
}
