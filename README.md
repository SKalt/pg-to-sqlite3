# pg-to-sqlite3

Load data from postgres to sqlite3 as fast as possible.

## Usage

Before using `pg-to-sqlite3`, test that your DDL is compatible with sqlite3:

```sh
#!/usr/bin/env bash
PGUSER="${PGUSERNAME:-}"
PGPASSWORD="${PGPASSWORD:-}"
PGHOST="${PGHOST:-}"
PGPORT="${PGPORT:-}"
PGDATABASE="${PGDATABASE:-}"

# TODO: automate this in, like, anything that's not bash
check_schema_compatible() {
  local ident; ident="$(date -Iseconds).sql"
  set -x;
  pg_dump \
    --no-tablespaces \
    --no-synchronized-snapshots \
    --no-security-labels \
    --no-subscriptions \
    --no-privileges \
    --schema-only |  sed '/^SET/d; /^SELECT pg_catalog/d;'
    > /tmp/$ident;
  sqlite3 /tmp/temp.db ".import /tmp/$ident";
}

check_compatible;
```
- (schemas: sqlite has `ATTACH`ed dbs, but attachments don't persist between connections)
- (roles)
- (users)
- [ extensions: need careful translation ]
- tables -> columns -> tables
- (partitions: see schemas)
- views
- constraints

Inspired by [`db-to-sqlite`][2] and [datasette][1].

[1]: https://datasette.io/
[2]: https://github.com/simonw/db-to-sqlite
[3]: https://github.com/astef/benchmark-sqlite3-bulk-insert
