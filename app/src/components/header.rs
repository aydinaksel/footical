use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn Header() -> impl IntoView {
    let location = use_location();

    let home_class = move || {
        if location.pathname.get() == "/" {
            "font-medium text-blue-600"
        } else {
            "font-medium text-gray-500 hover:text-gray-900"
        }
    };

    let calendar_class = move || {
        if location.pathname.get() == "/calendar" {
            "font-medium text-blue-600"
        } else {
            "font-medium text-gray-500 hover:text-gray-900"
        }
    };

    view! {
        <header class="bg-white shadow-sm border-b border-gray-200">
            <nav class="max-w-md mx-auto px-4 py-3 flex gap-6">
                <A href="/"><span class=home_class>"Home"</span></A>
                <A href="/calendar"><span class=calendar_class>"Calendar"</span></A>
            </nav>
        </header>
    }
}
