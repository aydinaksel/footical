use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::{use_cookie_with_options, SameSite, UseCookieOptions};

const TRACKED_TEAM_COOKIE_NAME: &str = "footical_team";
const TRACKED_TEAM_COOKIE_MAX_AGE_MILLISECONDS: i64 = 365 * 24 * 60 * 60 * 1000;

#[cfg(feature = "hydrate")]
const SUPERSEDED_TRACKED_TEAM_STORAGE_KEY: &str = "footical_team_id";

#[derive(Clone, Copy)]
pub struct TrackedTeam {
    pub team_id: Signal<Option<i32>>,
    pub set_team_id: WriteSignal<Option<i32>>,
}

pub fn provide_tracked_team() -> TrackedTeam {
    let (team_id, set_team_id) = use_cookie_with_options::<i32, JsonSerdeCodec>(
        TRACKED_TEAM_COOKIE_NAME,
        UseCookieOptions::default()
            .path("/")
            .same_site(SameSite::Lax)
            .max_age(TRACKED_TEAM_COOKIE_MAX_AGE_MILLISECONDS),
    );

    #[cfg(feature = "hydrate")]
    adopt_team_from_browser_storage(team_id, set_team_id);

    let tracked_team = TrackedTeam {
        team_id,
        set_team_id,
    };
    provide_context(tracked_team);
    tracked_team
}

#[cfg(feature = "hydrate")]
fn adopt_team_from_browser_storage(
    team_id: Signal<Option<i32>>,
    set_team_id: WriteSignal<Option<i32>>,
) {
    use leptos_use::storage::use_local_storage;

    let (stored_team_id, _set_stored_team_id, remove_stored_team_id) =
        use_local_storage::<Option<i32>, JsonSerdeCodec>(SUPERSEDED_TRACKED_TEAM_STORAGE_KEY);

    Effect::new(move |_| {
        if team_id.get().is_some() {
            return;
        }
        if let Some(previously_tracked_team_id) = stored_team_id.get() {
            set_team_id.set(Some(previously_tracked_team_id));
            remove_stored_team_id();
        }
    });
}

pub fn use_tracked_team() -> TrackedTeam {
    use_context::<TrackedTeam>().unwrap_or_else(provide_tracked_team)
}
