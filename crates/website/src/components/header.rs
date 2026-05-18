use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn Header() -> impl IntoView {
    let location = use_location();

    let nav_class = move |path: &'static str| {
        move || {
            if location.pathname.get() == path {
                "font-medium text-blue-600 cursor-pointer"
            } else {
                "font-medium text-gray-500 hover:text-gray-900 cursor-pointer"
            }
        }
    };

    view! {
        <header class="bg-white shadow-sm border-b border-gray-200">
            <nav class="max-w-md mx-auto px-4 py-3 flex gap-6">
                <A href="/"><span class=nav_class("/")>"Home"</span></A>
                <A href="/fixtures"><span class=nav_class("/fixtures")>"Fixtures"</span></A>
                <A href="/today"><span class=nav_class("/today")>"Today"</span></A>
                <span class="ml-auto">
                    <A href="/admin"><span class=nav_class("/admin")>"Admin"</span></A>
                </span>
            </nav>
        </header>
    }
}
