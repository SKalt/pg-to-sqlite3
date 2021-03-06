SELECT
  c.oid
  , c.relname AS name
  , c.relkind::TEXT
  , c.reltuples::BIGINT AS approx_n_rows
  , pg_catalog.pg_get_userbyid(c.relowner) as owner
FROM pg_catalog.pg_class c
    LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind IN ('r','v','m','s','p')
    AND n.nspname = $1
    AND pg_catalog.pg_table_is_visible(c.oid)
ORDER BY 1, 2;
