//! Read-only, policy-aware named query functions.
//!
//! This module is the default consumer's entire view of the database. Business
//! reads added here must carry tenant/user scope and apply redaction. Generated
//! entities and raw query builders stay private to this crate. Prefer
//! `get_published_items_for_tenant(tenant_id)`-style named contracts over
//! anything that hands a caller a query builder.

use crate::{
    connection::{inspect_connection, InternalConnectionState},
    OrmError, ReadContext, ORG_SCHEMA,
};

/// Safe, implementation-independent evidence about the active connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionState {
    schema: String,
    transaction_read_only: bool,
}

impl ConnectionState {
    pub(crate) fn from_internal(state: InternalConnectionState) -> Self {
        Self {
            schema: state.schema,
            transaction_read_only: state.transaction_read_only,
        }
    }

    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }

    #[must_use]
    pub fn transaction_read_only(&self) -> bool {
        self.transaction_read_only
    }
}

/// Reject policy drift even when the SeaORM and Diesel sessions drift together.
/// Both drivers agreeing on an unexpected schema is not proof of tenant isolation.
fn require_read_policy(state: &InternalConnectionState) -> Result<(), OrmError> {
    if state.schema != ORG_SCHEMA {
        return Err(OrmError::policy(format!(
            "read context resolved schema {:?}; expected {ORG_SCHEMA:?}",
            state.schema
        )));
    }
    if !state.transaction_read_only {
        return Err(OrmError::policy(
            "read context lost its read-only transaction policy",
        ));
    }
    Ok(())
}

/// Return verified policy state without exposing a SeaORM connection.
/// The two independent drivers must agree AND satisfy the canonical schema and
/// read-only policy. A shared drift cannot be reported as a healthy connection.
pub async fn connection_state(context: &ReadContext) -> Result<ConnectionState, OrmError> {
    let sea = inspect_connection(context.connection()).await?;
    let diesel = context.diesel_state().await?;
    if sea != diesel {
        return Err(OrmError::policy("Diesel and SeaORM read policies disagree"));
    }
    require_read_policy(&sea)?;
    Ok(ConnectionState::from_internal(sea))
}

/// Lightweight named readiness read for consumers and health checks.
pub async fn ping(context: &ReadContext) -> Result<(), OrmError> {
    connection_state(context).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(schema: &str, read_only: bool) -> InternalConnectionState {
        InternalConnectionState {
            schema: schema.to_owned(),
            transaction_read_only: read_only,
        }
    }

    #[test]
    fn expected_schema_and_read_only_policy_are_admitted() {
        assert!(require_read_policy(&state(ORG_SCHEMA, true)).is_ok());
    }

    #[test]
    fn both_drivers_cannot_collectively_drift_to_another_schema() {
        let result = require_read_policy(&state("public", true));
        assert!(matches!(result, Err(OrmError::PolicyViolation(_))));
    }

    #[test]
    fn canonical_schema_does_not_override_read_only_requirement() {
        let result = require_read_policy(&state(ORG_SCHEMA, false));
        assert!(matches!(result, Err(OrmError::PolicyViolation(_))));
    }
}
