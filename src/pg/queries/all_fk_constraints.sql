SELECT tc.constraint_name,
  tc.constraint_type,
  tc.table_name,
  array_agg(kcu.column_name::TEXT) AS columns,
  ccu.table_name AS foreign_table_name,
  array_agg(ccu.column_name::TEXT) AS foreign_columns,
  tc.is_deferrable,
  tc.initially_deferred
FROM information_schema.table_constraints AS tc
  JOIN information_schema.key_column_usage AS kcu
    ON tc.constraint_name = kcu.constraint_name
  JOIN information_schema.constraint_column_usage AS ccu
    ON tc.constraint_name = ccu.constraint_name
WHERE tc.constraint_schema = $1 AND tc.constraint_type = 'FOREIGN KEY'
GROUP BY tc.constraint_name,
  tc.constraint_type,
  tc.table_name,
  tc.is_deferrable,
  tc.initially_deferred,
  ccu.table_name
        