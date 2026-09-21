use leptos::prelude::*;
use tracing::{event, Level};

pub fn database_pool() -> Result<sqlx::SqlitePool, ServerFnError> {
    use_context::<sqlx::SqlitePool>().ok_or_else(|| {
        event!(
            name: "database.pool.missing",
            Level::ERROR,
            error.type = "missing_context",
            "no database pool in context",
        );
        ServerFnError::new("the database is unavailable")
    })
}

pub fn report_query_failure(operation: &'static str, error: sqlx::Error) -> ServerFnError {
    event!(
        name: "database.query.failure",
        Level::ERROR,
        db.system.name = "sqlite",
        db.operation.name = operation,
        error.type = "sqlx",
        error.message = %error,
        "{{db.operation.name}} failed: {{error.message}}",
    );
    ServerFnError::new(format!("{operation} failed"))
}
