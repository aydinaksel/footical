use crate::server::auth::check_auth;
use leptos::prelude::*;
use leptos_router::components::Redirect;

#[component]
pub fn AdminOnly(children: ChildrenFn) -> impl IntoView {
    let admin_session = Resource::new_blocking(|| (), |_| check_auth());

    view! {
        <Suspense fallback=|| ()>
            {move || {
                let children = children.clone();
                Suspend::new(async move {
                    match admin_session.await {
                        Ok(true) => children().into_any(),
                        _ => view! { <Redirect path="/admin/login" /> }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
    .into_any()
}
