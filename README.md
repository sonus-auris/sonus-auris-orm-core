# sonus-auris-orm-core

Canonical **SeaORM** boundary for the `sonus-auris` organization. It is the only repository that may publish the shared Sonus Auris ORM package; `sonus-auris-lib` may consume or re-export it temporarily but must not define a second authoritative ORM crate.

Governed by [`sonus-auris/.github/SERVICE_AND_DATA_ARCHITECTURE.md`](https://github.com/sonus-auris/.github/blob/main/SERVICE_AND_DATA_ARCHITECTURE.md).

## Contract

| Consumer | Feature | Public surface |
| --- | --- | --- |
| Web/default consumer | `read-only` (default) | `ReadContext`, role-aware connection, and named functions under `read` |
| API server | `read-write` | Adds `WriteContext` and named functions under `write` |

Raw SeaORM/SQLx connections, entity managers, query builders, and backend error types stay private. A default consumer cannot import `WriteContext`, `connect_read_write`, or the `write` module; a compile-fail doctest enforces that boundary.

`connect_read_only` pins `search_path=sonus_auris`, sets `default_transaction_read_only=on` in the PostgreSQL startup packet, and verifies both settings before returning an opaque context. `connect_read_write` is compiled only with `read-write` and rejects a transaction-read-only session.

## Shared schema source

Schema definitions come from [`ORESoftware/k8s-libs-and-shared-defs`](https://github.com/ORESoftware/k8s-libs-and-shared-defs), never from independently authored entities here. [`shared-defs.lock.json`](shared-defs.lock.json) pins revision `c8bdc06d74746acc6439f9527ebd02697fdf028b`, organization slice `sonus-auris`, schema `sonus_auris`, and the generated Rust SeaORM adapter path.

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
