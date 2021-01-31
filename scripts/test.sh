#!/bin/sh
if [ -e ./temp.db ]; then rm ./temp.db; fi
make build
./target/debug/pg-to-sqlite3 \
  --src 'postgres://postgres:password@0.0.0.0:5432' \
  --dest temp.db \
  --no-views
# sqlite3 ./temp.db 'select * from sqlite_master'
