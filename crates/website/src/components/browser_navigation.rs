pub fn reload_page() {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            let _reload_started = window.location().reload();
        }
    }
}

pub fn navigate_to(path: &str) {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            let _navigation_started = window.location().set_href(path);
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _unused_on_the_server = path;
    }
}
