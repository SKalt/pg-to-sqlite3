SELECT
  col.column_name
  , col.ordinal_position
  , col.table_name
  , col.udt_name
  , col.is_nullable
  , col.character_maximum_length
  , col.character_octet_length
  , col.numeric_precision
  , col.numeric_precision_radix
  , col.numeric_scale
  , col.datetime_precision
  , col.interval_type
  , col.interval_precision
FROM information_schema.columns col
WHERE col.table_schema = $1 AND col.table_name = ANY($2)
ORDER BY 2
        