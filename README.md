# sonus-auris-orm-core

Opaque Rust persistence boundary for the `sonus-auris` organization. This
repository is a derived runtime package: it is not an independently editable
schema, SQL, or migration authority.

The current implementation is a SeaORM-based read-only/read-write shell. The
target described here is **not yet a completed cutover**: Diesel plus
`diesel-async` becomes the primary Rust data path, while SeaORM remains a
secondary, DB-first representation used for independent parity checks and any
named operations that deliberately remain on SeaORM.

The authored persistence sources, PostgreSQL extensions, desired SQL, and
generation evidence belong in
[`sonus-auris-lib-core`](https://github.com/sonus-auris/sonus-auris-lib-core).
See its `docs/dual-source-persistence.md` for the complete target pipeline.

## Non-negotiable namespace gate

The inherited shared SQL currently creates unqualified `sound_recorder_*`
tables, which resolve to `public` under the shared-platform convention. This
crate currently pins `search_path=sonus_auris`, and
[`shared-defs.lock.json`](shared-defs.lock.json) records that same dedicated
schema name. Those claims conflict.

Do not publish generated entities or change the connection code by guessing
which namespace is intended. Before runtime cutover, the migration job must
dump the live catalog and provider migration ledger, identify the real
namespace, and record the approved decision in lib-core. Moving an existing
table from `public` to `sonus_auris` is a separate expand/contract migration,
not a side effect of extracting ownership from the shared repository.

## Boundary contract

| Consumer | Capability | Public surface |
| --- | --- | --- |
| Web/default consumer | `read-only` | Opaque read context and named, owner-scoped reads |
| API server | `read-write` | Opaque write context and named product operations |
| Admin server | Separate admin capability | Explicitly approved admin operations; never inherited through the public runtime role |
| Migration job | None from this crate | Uses the lib-core desired release and `dpm` with a dedicated DDL principal |

Raw Diesel connections, SeaORM/SQLx connections, query builders, generated
entities, and backend errors remain private. Cargo feature selection expresses
API intent; it is not the security boundary. PostgreSQL roles, grants, RLS,
separate database identities, and audited runtime connection checks enforce the
boundary.

No server may run DDL at startup. Only the serialized migration job receives
DDL privileges.

## Dual-source release provenance

Every releasable version of this crate must identify one immutable,
jointly-certified `sonus-auris-lib-core` release containing:

- the co-equal, independently authored TypeSpec and JSON Schema/OpenAPI trees;
- the common authored PostgreSQL extension bundle for RLS, policies,
  functions, triggers, indexes, grants, and provider-specific behavior;
- normalized peer-source, catalog, ORM, behavioral, and wire parity
  reports;
- the reviewed desired SQL digest;
- Diesel and SeaORM generation manifests; and
- the compatible migration window and required database capabilities.

The production graph never generates either authored source from the other.
Optional cross-translations are diagnostic witnesses only; they cannot feed
SQL, Protobuf, OpenAPI, interfaces, clients, ORM code, migration artifacts, or
a release. Either authored source vetoes a release when a semantic mismatch is
unexplained.

The existing `shared-defs.lock.json` is retained as historical provenance for
the pre-extraction baseline. It must not remain a dependency or the source of a
new release after Sonus lib-core publishes the replacement artifacts.

## Diesel primary, SeaORM secondary

The target generation sequence is:

```text
TypeSpec lane            -> SQL A + Diesel A + IR A + Protobuf/gRPC
JSON Schema/OpenAPI lane -> SQL B + Diesel B + IR B + interfaces/clients

SQL A + extension E -> scratch PostgreSQL A -> sea-orm-cli -> SeaORM A
SQL B + extension E -> scratch PostgreSQL B -> sea-orm-cli -> SeaORM B

normalized(A) == normalized(B) == reviewed desired release
```

Diesel is primary because its explicit Rust schema/model layer can drive the
main compile-time checked runtime and can assist with reviewed schema-diff
migration drafts. Diesel diff output is not complete PostgreSQL DDL: defaults,
custom checks, RLS policies, guard functions, grants, and similar extensions
remain authored in lib-core and are reviewed as part of the desired release.

SeaORM is secondary because `sea-orm-cli` reads a database and generates
entities; it does not turn Rust entities into the complete desired DDL. Running
it against both scratch databases gives an independent DB-readback check. The
normalized entity manifests must agree before a release, but consumers still
receive named opaque operations instead of either ORM's generated surface.

## Zed dependency graph

The desired Zed relationship is:

```text
sonus-auris-lib-core@immutable-release
        -> sonus-auris-orm-core@immutable-release
        -> web/api/admin consumers by explicit capability
```

`sonus-auris-orm-core` must pin the exact lib-core release and artifact
digests. It must not import SQL or generated ORM artifacts from
`ORESoftware/k8s-libs-and-shared-defs`. Candidate artifacts live only in CI;
only a parity-clean, reviewed release may enter the Zed dependency graph.

## Migrations

`sonus-auris-infra` is the sole migration-execution owner. Its org-fixed,
non-secret target manifest pins the matching lib-core desired SQL release and
DPM version through Zed, resolves direct/unpooled connection references, keeps
read-only planning and DDL apply identities separate, obtains the org/schema
lock, and publishes the plan/apply/convergence receipt. The standard
`db-plan`, `db-verify`, `db-apply`, and `db-status` entry points use a
flags-2-env-governed Rust wrapper; credentials are runtime-injected and never
passed as flags.

`declarative-migrations`/`dpm` compares that certified release with a fresh live
catalog dump, produces a reviewable plan, verifies it against a shadow
database, requires the reviewed digest and unchanged live fingerprint before
apply, and verifies an empty residual diff. Shared-platform plans fail closed
if they touch objects outside the declared Sonus schema and dependency closure.
This crate exposes neither DPM nor a migration credential; it only provides a
read-only compatibility/readiness check for the expected schema release.

Data backfills, ownership and role changes, provider migration ledgers, and
other behavior outside declarative DDL require explicit companion steps. A
clean structural diff alone is not sufficient release evidence.

## Current implementation gap

The read/write feature boundary and startup-session checks exist today. The
following work is still required before this document describes released
runtime behavior:

- resolve the `public` versus `sonus_auris` namespace contradiction from live
  evidence;
- publish the dual-source lib-core artifacts and parity reports;
- replace the shared-definitions Zed dependency with the org-owned lib-core
  release;
- generate, normalize, and review both Diesel and SeaORM candidates;
- replace generic connection-state probes with named Sonus operations; and
- run PostgreSQL and CockroachDB compatibility lanes with real least-privilege
  roles.

## Validation

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo test --all-targets --all-features
cargo test --doc
```

A live denial probe is intentionally ignored by default because it attempts
forbidden DDL against a disposable database:

```sh
ORM_CORE_TEST_DATABASE_URL='postgres://sonus_web_ro@localhost/sonus_test' \
  cargo test live_read_only_context_rejects_schema_ddl -- --ignored
```

## References

- [Diesel schema-diff migration generation](https://diesel.rs/news/2_1_0_release.html)
- [SeaORM database-to-entity generation](https://www.sea-ql.org/SeaORM/docs/0.12.x/generate-entity/sea-orm-cli/)
- [TypeSpec custom emitters](https://typespec.io/docs/extending-typespec/emitters-basics/)
- [Declarative Migrations](https://github.com/declarative-migrations/declarative-postgres-migrate.rs)
