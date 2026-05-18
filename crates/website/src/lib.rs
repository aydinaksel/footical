pub mod app;
#[cfg(feature = "hydrate")]
mod clipboard;
mod components;
mod pages;
pub mod server;
mod types;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
