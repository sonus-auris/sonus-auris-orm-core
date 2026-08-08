/// PostgreSQL/CockroachDB schema owned by the Sonus Auris service boundary.
pub const ORG_SCHEMA: &str = "sonus_auris";

/// Organization slice consumed from the canonical shared-definitions repo.
pub const SHARED_DEFS_ORG_SLICE: &str = "sonus-auris";

/// Exact reviewed shared-definitions revision for generated entity input.
pub const SHARED_DEFS_REVISION: &str = "c8bdc06d74746acc6439f9527ebd02697fdf028b";

/// Generated adapter location within the shared-definitions repository.
pub const SHARED_DEFS_SEA_ORM_ADAPTER: &str = "pg-defs/generated/rust/sea-orm";
