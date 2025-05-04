### CQL

Suggested cql schemas should be DDL (_Data Definition Language_) schema (e.g. `CREATE TABLE`, `ALTER TABLE`,
`CREATE KEYSPACE`, `DELETE`, e.t.c). It's recommended to only add DDL statements in This directory.

For DML (_Data Manipulation Language_) statements, these should be defined as strings within the Rust code because
you'll typically use them with prepared statements.
