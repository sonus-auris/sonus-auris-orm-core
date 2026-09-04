//! Private Diesel companion to the existing SeaORM read capability.

use std::{sync::Arc, time::Duration};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    sql_types::{Nullable, Text},
    PgConnection, RunQueryDsl,
};
use tokio::sync::Semaphore;

use crate::{connection::InternalConnectionState, OrmError, ORG_SCHEMA};

#[derive(Clone)]
pub(crate) struct DieselReadPool {
    pool: Pool<ConnectionManager<PgConnection>>,
    permit: Arc<Semaphore>,
    timeout: Duration,
}

#[derive(diesel::QueryableByName)]
struct StateRow {
    #[diesel(sql_type = Nullable<Text>)]
    schema_name: Option<String>,
    #[diesel(sql_type = Text)]
    transaction_read_only: String,
}

impl DieselReadPool {
    pub(crate) async fn connect(url: &str, timeout: Duration) -> Result<Self, OrmError> {
        if timeout.is_zero() {
            return Err(OrmError::policy(
                "Diesel acquisition timeout must be positive",
            ));
        }
        let result = Self {
            pool: Pool::builder()
                .max_size(1)
                .min_idle(Some(0))
                .connection_timeout(timeout)
                .build_unchecked(ConnectionManager::new(url)),
            permit: Arc::new(Semaphore::new(1)),
            timeout,
        };
        result.inspect().await?;
        Ok(result)
    }

    pub(crate) async fn inspect(&self) -> Result<InternalConnectionState, OrmError> {
        let permit = tokio::time::timeout(self.timeout, self.permit.clone().acquire_owned())
            .await
            .map_err(|_| unavailable())?
            .map_err(|_| unavailable())?;
        let pool = self.pool.clone();
        let timeout = self.timeout;
        tokio::task::spawn_blocking(move || {
            // Cancellation cannot release the permit before this worker finishes.
            let _permit = permit;
            let mut connection = pool.get_timeout(timeout).map_err(|_| unavailable())?;
            diesel::sql_query(
                "SELECT set_config('search_path', $1, false), \
                 set_config('default_transaction_read_only', 'on', false), \
                 set_config('statement_timeout', '5000', false)",
            )
            .bind::<Text, _>(ORG_SCHEMA)
            .execute(&mut connection)
            .map_err(|_| unavailable())?;
            let row = diesel::sql_query(
                "SELECT current_schema() AS schema_name, \
                 current_setting('default_transaction_read_only') AS transaction_read_only",
            )
            .get_result::<StateRow>(&mut connection)
            .map_err(|_| unavailable())?;
            verify(row)
        })
        .await
        .map_err(|_| unavailable())?
    }
}

fn verify(row: StateRow) -> Result<InternalConnectionState, OrmError> {
    if row.schema_name.as_deref() != Some(ORG_SCHEMA) || row.transaction_read_only != "on" {
        return Err(OrmError::policy(
            "Diesel schema or read-only policy does not match",
        ));
    }
    Ok(InternalConnectionState {
        schema: ORG_SCHEMA.into(),
        transaction_read_only: true,
    })
}

fn unavailable() -> OrmError {
    // Driver diagnostics can contain URLs or server-provided identity data.
    OrmError::database("Diesel read connection unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_policy_rejects_missing_wrong_or_writable_schema() {
        for (schema_name, transaction_read_only) in [
            (None, "on"),
            (Some("wrong".into()), "on"),
            (Some(ORG_SCHEMA.into()), "off"),
        ] {
            assert!(verify(StateRow {
                schema_name,
                transaction_read_only: transaction_read_only.into()
            })
            .is_err());
        }
        assert!(verify(StateRow {
            schema_name: Some(ORG_SCHEMA.into()),
            transaction_read_only: "on".into()
        })
        .is_ok());
    }

    #[tokio::test]
    #[ignore = "requires dedicated ORM_CORE_TEST_DATABASE_URL and the org schema"]
    async fn live_both_drivers_agree_on_read_policy() {
        let url = std::env::var("ORM_CORE_TEST_DATABASE_URL").expect("test database URL");
        let context = crate::connect_read_only(&url).await.unwrap();
        let state = crate::read::connection_state(&context).await.unwrap();
        assert_eq!(state.schema(), ORG_SCHEMA);
        assert!(state.transaction_read_only());
    }
}
