SELECT DISTINCT
  source_rel.oid        AS source_oid
  , source_rel.relname    AS source_table
  , dependent_rel.relname AS dependent_rel
  , dependent_rel.oid     AS dependent_oid
FROM pg_catalog.pg_depend AS dep
JOIN pg_catalog.pg_rewrite AS rewrite ON dep.objid = rewrite.oid
JOIN pg_catalog.pg_class AS dependent_rel ON rewrite.ev_class = dependent_rel.oid
JOIN pg_catalog.pg_class AS source_rel ON dep.refobjid = source_rel.oid
JOIN pg_catalog.pg_namespace source_ns ON source_ns.oid = source_rel.relnamespace
WHERE source_ns.nspname = $1 AND source_rel.oid <> dependent_rel.oid
