SELECT (c.relname)::information_schema.sql_identifier AS name,
pg_get_viewdef(c.oid)::information_schema.character_data AS defn
FROM pg_catalog.pg_class AS c 
WHERE c.oid = ANY($1);
