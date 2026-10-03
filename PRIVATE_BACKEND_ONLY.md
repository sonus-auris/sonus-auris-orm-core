# Private backend only

Executable ORM/database capability code in this repository is backend-only. Product schema authority remains in `sonus-auris/sonus-auris-interfaces`; shared ORESoftware interface/validator repositories provide validation and audit tooling only.

Keep database connections opaque, default to read-only capability, require explicit write capability, and keep migration/DDL execution outside application ORM startup.

The GitHub repository itself must be private before this boundary is considered fully conformant.
