SELECT
  tc.constraint_name
  , tc.table_name
  , array_agg(kcu.column_name::TEXT) AS columns
  , tc.is_deferrable
  , tc.initially_deferred
FROM information_schema.table_constraints AS tc
  JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
WHERE tc.constraint_schema = $1 AND tc.constraint_type = 'UNIQUE'
GROUP BY tc.constraint_name,
  tc.constraint_type,
  tc.table_name,
  tc.is_deferrable,
  tc.initially_deferred;
