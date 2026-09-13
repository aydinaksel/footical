use crate::types::{FineType, LedgerEntry, PlayerBalance, SquadFixture, SquadPlayer};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
fn database_pool() -> Result<sqlx::SqlitePool, ServerFnError> {
    use_context::<sqlx::SqlitePool>().ok_or_else(|| ServerFnError::new("no database pool"))
}

#[cfg(feature = "ssr")]
async fn require_admin() -> Result<(), ServerFnError> {
    if crate::server::auth::check_auth().await? {
        Ok(())
    } else {
        Err(ServerFnError::new("not authorised"))
    }
}

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
    .map_err(|error| ServerFnError::new(error.to_string()))
}

#[server]
pub async fn get_squad_fixtures() -> Result<Vec<SquadFixture>, ServerFnError> {
    let pool = database_pool()?;
    sqlx::query_as::<_, SquadFixture>(
        "SELECT squad_fixture_id, competition, kicks_off_at, is_home, opponent, venue
         FROM squad_fixture
         WHERE kicks_off_at >= datetime('now', '-3 hours')
         ORDER BY kicks_off_at",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))
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
    .map_err(|error| ServerFnError::new(error.to_string()))
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
    .map_err(|error| ServerFnError::new(error.to_string()))
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
    .map_err(|error| ServerFnError::new(error.to_string()))?;

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
    .map_err(|error| ServerFnError::new(error.to_string()))?;

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
        .map_err(|error| ServerFnError::new(error.to_string()))
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
        .map_err(|error| ServerFnError::new(error.to_string()))
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
        .map_err(|error| ServerFnError::new(error.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("entry not found"));
    }

    Ok(())
}
