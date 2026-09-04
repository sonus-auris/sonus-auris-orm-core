# sonus-auris-orm-core

Read contexts now use both SeaORM and a private Diesel companion. The Diesel
pool has one connection in addition to the configured SeaORM pool. Acquisition
is bounded, synchronous work runs on a blocking worker, and cancellation keeps
its permit until that worker finishes. Both drivers install the same schema and
read-only session policy; `read::connection_state` and `read::ping` reject any
disagreement. Neither driver exposes a connection to the consumer.

Run the ignored `live_both_drivers_agree_on_read_policy` test with
`ORM_CORE_TEST_DATABASE_URL` targeting a disposable database containing the org
schema. Native builds need PostgreSQL's libpq development library.

The Zed manifest declares this organization's interfaces and lib-core. Frozen
artifact resolution remains a separate release gate while the registry is
unavailable; these declarations do not certify a successful Zed installation.

Canonical SeaORM and Diesel boundary for the `sonus-auris` organization.
The requested dependency direction is orm-core importing shared lib-core and
all four servers importing both; lib-core must not import orm-core. Existing
business operations and consumer dependencies still need migration before
publishing a replacement package.

Governed by [`sonus-auris/.github/SERVICE_AND_DATA_ARCHITECTURE.md`](https://github.com/sonus-auris/.github/blob/main/SERVICE_AND_DATA_ARCHITECTURE.md).

## Contract

| Consumer | Feature | Public surface |
| --- | --- | --- |
| Web/default consumer | `read-only` (default) | `ReadContext`, role-aware connection, and named functions under `read` |
| API server | `read-write` | Adds `WriteContext` and named functions under `write` |

Raw SeaORM/SQLx connections, entity managers, query builders, and backend error types stay private. A default consumer cannot import `WriteContext`, `connect_read_write`, or the `write` module; a compile-fail doctest enforces that. Note this is an intent-and-ergonomics boundary, not a security one: Cargo feature resolution is additive, so any crate in a consumer's graph that enables `read-write` turns those symbols on. The authoritative control is the SELECT-only database role.

`connect_read_only` pins `search_path=sonus_auris`, sets `default_transaction_read_only=on` in the PostgreSQL startup packet, and verifies both settings before returning an opaque context. `connect_read_write` is compiled only with `read-write` and rejects a transaction-read-only session.

## Shared schema source

Schema definitions come from [`ORESoftware/k8s-libs-and-shared-defs`](https://github.com/ORESoftware/k8s-libs-and-shared-defs), never from independently authored entities here. [`shared-defs.lock.json`](shared-defs.lock.json) pins revision `c8bdc06d74746acc6439f9527ebd02697fdf028b`, organization slice `sonus-auris`, schema `sonus_auris`, and the generated Rust SeaORM adapter path.

Each release pins the exact shared-definition revision/digest it was generated against; a major version bump is treated as a schema event and participates in the expand/contract compatibility window. The crate targets PostgreSQL and CockroachDB (postgres wire protocol) through SeaORM's `sqlx-postgres` backend, but a shared codebase does not make the engines behave identically — engine-specific behavior, notably retryable serialization errors, must be tested per engine.

The connection and feature boundary is implemented now. Importing the generated Sonus Auris entity slice and replacing the generic connection-state reads with business-specific named queries remains a merge gate; do not expose the generated crate wholesale to consumers.

## Usage

Default web/read consumer:

```toml
sonus-auris-orm-core = { git = "https://github.com/sonus-auris/sonus-auris-orm-core.git", rev = "<merge-commit>" }
```

```rust,no_run
use sonus_auris_orm_core::{connect_read_only, read};

# async fn example() -> Result<(), sonus_auris_orm_core::OrmError> {
let context = connect_read_only("postgres://sonus_web_ro@db/sonus").await?;
read::ping(&context).await?;
# Ok(())
# }
```

API/write consumer:

```toml
sonus-auris-orm-core = {
  git = "https://github.com/sonus-auris/sonus-auris-orm-core.git",
  rev = "<merge-commit>",
  default-features = false,
  features = ["read-write"]
}
```

```rust,no_run
use sonus_auris_orm_core::{connect_read_write, write};

# async fn example() -> Result<(), sonus_auris_orm_core::OrmError> {
let context = connect_read_write("postgres://sonus_api_rw@db/sonus").await?;
write::ping(&context).await?;
# Ok(())
# }
```

## Migrations

There is no migration tooling in this crate. The Sonus Auris API server owns compatibility requirements, and a separate `declarative-migrations`/`dpm` release job applies reviewed DDL with the project-scoped migrator identity. Runtime API and web identities do not receive DDL rights.

## Validation

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo test --all-targets --all-features
cargo test --doc
```

A live denial probe is included but ignored by default because it performs an intentionally forbidden DDL statement against a disposable database:

```sh
ORM_CORE_TEST_DATABASE_URL='postgres://sonus_web_ro@localhost/sonus_test' \
  cargo test live_read_only_context_rejects_schema_ddl -- --ignored
```

Run that lane against both PostgreSQL and CockroachDB with a real SELECT-only web principal before releasing a consumer pin.
