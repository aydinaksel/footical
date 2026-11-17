use crate::types::{
    FineType, LedgerEntry, PlayerBalance, SquadFixture, SquadPlayer, SquadRosterEntry,
};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::server::auth::require_admin;
#[cfg(feature = "ssr")]
use crate::server::database::{database_pool, report_query_failure};

#[server]
pub async fn get_player_balances() -> Result<Vec<PlayerBalance>, ServerFnError> {
    let pool = database_pool()?;
    sqlx::query_as::<_, PlayerBalance>(
        "SELECT
             squad_player.squad_player_id,
             squad_player.name,
             COALESCE(fine_totals.total_pence, 0) AS fines_pence,
             COALESCE(payment_totals.total_pence, 0) AS payments_pence,
             COALESCE(fine_totals.total_pence, 0)
                 - COALESCE(payment_totals.total_pence, 0) AS balance_pence
         FROM squad_player
         LEFT JOIN (
             SELECT squad_player_id, SUM(amount_pence) AS total_pence
             FROM fine GROUP BY squad_player_id
         ) AS fine_totals ON fine_totals.squad_player_id = squad_player.squad_player_id
         LEFT JOIN (
             SELECT squad_player_id, SUM(amount_pence) AS total_pence
             FROM payment GROUP BY squad_player_id
         ) AS payment_totals ON payment_totals.squad_player_id = squad_player.squad_player_id
         WHERE squad_player.is_active = 1
         ORDER BY squad_player.name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_player_balances", error))
}

#[server]
pub async fn get_squad_fixtures() -> Result<Vec<SquadFixture>, ServerFnError> {
    let pool = database_pool()?;
    sqlx::query_as::<_, SquadFixture>(
        "SELECT fixture.fixture_id,
                competition.name AS competition,
                fixture.scheduled_at AS kicks_off_at,
                fixture.home_team_id = squad_team.team_id AS is_home,
                opponent.name AS opponent,
                venue.name AS venue,
                fixture.status = 'postponed' AS is_postponed
         FROM team AS squad_team
         JOIN source ON source.source_id = squad_team.source_id
         JOIN fixture
           ON squad_team.team_id IN (fixture.home_team_id, fixture.away_team_id)
         JOIN team AS opponent
           ON opponent.team_id = CASE
               WHEN fixture.home_team_id = squad_team.team_id THEN fixture.away_team_id
               ELSE fixture.home_team_id
           END
         JOIN competition ON competition.competition_id = fixture.competition_id
         LEFT JOIN venue ON venue.venue_id = fixture.venue_id
         WHERE source.code = 'fa-fulltime'
           AND squad_team.source_key = ?
           AND fixture.status <> 'removed'
           AND fixture.scheduled_at >= strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime', '-3 hours')
         ORDER BY fixture.scheduled_at",
    )
    .bind(footical_scraper::squad_team_key())
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_squad_fixtures", error))
}

#[server]
pub async fn get_squad_players() -> Result<Vec<SquadPlayer>, ServerFnError> {
    let pool = database_pool()?;
    sqlx::query_as::<_, SquadPlayer>(
        "SELECT squad_player_id, name FROM squad_player
         WHERE is_active = 1 ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_squad_players", error))
}

#[server]
pub async fn get_fine_types() -> Result<Vec<FineType>, ServerFnError> {
    let pool = database_pool()?;
    sqlx::query_as::<_, FineType>(
        "SELECT fine_type_id, name, default_amount_pence FROM fine_type
         WHERE is_active = 1 ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_fine_types", error))
}

#[server]
pub async fn record_fine(
    squad_player_id: i32,
    fine_type_id: i32,
    note: String,
) -> Result<(), ServerFnError> {
    require_admin().await?;

    let pool = database_pool()?;
    let trimmed_note = note.trim();
    let result = sqlx::query(
        "INSERT INTO fine
             (squad_player_id, fine_type_id, amount_pence, incurred_on, note)
         SELECT ?, fine_type_id, default_amount_pence, date('now'), ?
         FROM fine_type WHERE fine_type_id = ? AND is_active = 1",
    )
    .bind(squad_player_id)
    .bind(if trimmed_note.is_empty() {
        None
    } else {
        Some(trimmed_note)
    })
    .bind(fine_type_id)
    .execute(&pool)
    .await
    .map_err(|error| report_query_failure("insert_fine", error))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("unknown fine type"));
    }

    Ok(())
}

#[server]
pub async fn record_payment(
    squad_player_id: i32,
    amount_pence: i64,
    note: String,
) -> Result<(), ServerFnError> {
    require_admin().await?;

    if amount_pence <= 0 {
        return Err(ServerFnError::new("amount must be positive"));
    }

    let pool = database_pool()?;
    let trimmed_note = note.trim();
    sqlx::query(
        "INSERT INTO payment (squad_player_id, amount_pence, paid_on, note)
         VALUES (?, ?, date('now'), ?)",
    )
    .bind(squad_player_id)
    .bind(amount_pence)
    .bind(if trimmed_note.is_empty() {
        None
    } else {
        Some(trimmed_note)
    })
    .execute(&pool)
    .await
    .map_err(|error| report_query_failure("insert_payment", error))?;

    Ok(())
}

#[cfg(feature = "ssr")]
const LEDGER_SELECT: &str = "SELECT
         fine.fine_id AS entry_id,
         0 AS is_payment,
         squad_player.name AS player_name,
         fine_type.name AS description,
         fine.amount_pence,
         fine.incurred_on AS happened_on,
         fine.note
     FROM fine
     JOIN fine_type ON fine_type.fine_type_id = fine.fine_type_id
     JOIN squad_player ON squad_player.squad_player_id = fine.squad_player_id
     WHERE (? IS NULL OR fine.squad_player_id = ?)
     UNION ALL
     SELECT
         payment.payment_id,
         1,
         squad_player.name,
         'Payment',
         payment.amount_pence,
         payment.paid_on,
         payment.note
     FROM payment
     JOIN squad_player ON squad_player.squad_player_id = payment.squad_player_id
     WHERE (? IS NULL OR payment.squad_player_id = ?)
     ORDER BY happened_on DESC, entry_id DESC";

#[server]
pub async fn get_player_ledger(squad_player_id: i32) -> Result<Vec<LedgerEntry>, ServerFnError> {
    let pool = database_pool()?;
    sqlx::query_as::<_, LedgerEntry>(LEDGER_SELECT)
        .bind(squad_player_id)
        .bind(squad_player_id)
        .bind(squad_player_id)
        .bind(squad_player_id)
        .fetch_all(&pool)
        .await
        .map_err(|error| report_query_failure("select_player_ledger", error))
}

#[server]
pub async fn get_recent_entries() -> Result<Vec<LedgerEntry>, ServerFnError> {
    require_admin().await?;
    let pool = database_pool()?;
    let statement = format!("{LEDGER_SELECT} LIMIT 20");
    sqlx::query_as::<_, LedgerEntry>(sqlx::AssertSqlSafe(statement))
        .bind(Option::<i32>::None)
        .bind(Option::<i32>::None)
        .bind(Option::<i32>::None)
        .bind(Option::<i32>::None)
        .fetch_all(&pool)
        .await
        .map_err(|error| report_query_failure("select_recent_entries", error))
}

#[server]
pub async fn delete_entry(entry_id: i32, is_payment: bool) -> Result<(), ServerFnError> {
    require_admin().await?;
    let pool = database_pool()?;

    let statement = if is_payment {
        "DELETE FROM payment WHERE payment_id = ?"
    } else {
        "DELETE FROM fine WHERE fine_id = ?"
    };

    let result = sqlx::query(statement)
        .bind(entry_id)
        .execute(&pool)
        .await
        .map_err(|error| report_query_failure("delete_ledger_entry", error))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("entry not found"));
    }

    Ok(())
}

#[server]
pub async fn get_squad_roster() -> Result<Vec<SquadRosterEntry>, ServerFnError> {
    require_admin().await?;
    let pool = database_pool()?;
    sqlx::query_as::<_, SquadRosterEntry>(
        "SELECT
             squad_player.squad_player_id,
             squad_player.name,
             squad_player.is_active,
             COALESCE(fine_totals.total_pence, 0)
                 - COALESCE(payment_totals.total_pence, 0) AS balance_pence
         FROM squad_player
         LEFT JOIN (
             SELECT squad_player_id, SUM(amount_pence) AS total_pence
             FROM fine GROUP BY squad_player_id
         ) AS fine_totals ON fine_totals.squad_player_id = squad_player.squad_player_id
         LEFT JOIN (
             SELECT squad_player_id, SUM(amount_pence) AS total_pence
             FROM payment GROUP BY squad_player_id
         ) AS payment_totals ON payment_totals.squad_player_id = squad_player.squad_player_id
         ORDER BY squad_player.is_active DESC, squad_player.name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_squad_roster", error))
}

#[server]
pub async fn set_player_active(squad_player_id: i32, is_active: bool) -> Result<(), ServerFnError> {
    require_admin().await?;
    let pool = database_pool()?;

    let result = sqlx::query("UPDATE squad_player SET is_active = ? WHERE squad_player_id = ?")
        .bind(is_active)
        .bind(squad_player_id)
        .execute(&pool)
        .await
        .map_err(|error| report_query_failure("update_player_active", error))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("unknown player"));
    }

    Ok(())
}

#[server]
pub async fn add_squad_player(name: String) -> Result<(), ServerFnError> {
    require_admin().await?;

    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err(ServerFnError::new("a player needs a name"));
    }

    let pool = database_pool()?;
    let result = sqlx::query(
        "INSERT INTO squad_player (name) VALUES (?)
         ON CONFLICT (name) DO UPDATE SET is_active = 1
         WHERE squad_player.is_active = 0",
    )
    .bind(trimmed_name)
    .execute(&pool)
    .await
    .map_err(|error| report_query_failure("insert_squad_player", error))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("that player is already in the squad"));
    }

    Ok(())
}
