use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;

const TRACKED_TEAM_STORAGE_KEY: &str = "footical_team_id";

#[derive(Clone, Copy)]
pub struct TrackedTeam {
    pub team_id: Signal<Option<i32>>,
    pub set_team_id: WriteSignal<Option<i32>>,
}

pub fn provide_tracked_team() -> TrackedTeam {
    let (team_id, set_team_id, _remove_from_storage) =
        use_local_storage::<Option<i32>, JsonSerdeCodec>(TRACKED_TEAM_STORAGE_KEY);
    let tracked_team = TrackedTeam {
        team_id,
        set_team_id,
    };
    provide_context(tracked_team);
    tracked_team
}

pub fn use_tracked_team() -> TrackedTeam {
    use_context::<TrackedTeam>().unwrap_or_else(provide_tracked_team)
}
