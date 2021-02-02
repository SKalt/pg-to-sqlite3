https://wiki.postgresql.org/wiki/Pg_depend_display
https://doxygen.postgresql.org/pg__dump_8c.html#a4ef904e6ea0a938a1fd5ec546dc920ac
https://sigterm.sh/2010/07/09/generating-a-dependency-graph-for-a-postgresql-database/
https://docs.rs/petgraph/0.5.1/petgraph/algo/fn.toposort.html

information_schema tables of note:

- information_schema.schemata
- information_schema.check_constraints
- information_schema.columns
- information_schema.key_column_usage
- information_schema.referential_contraints
- information_schema.table_constraints
- information_schema.views

less portable, but still interesting:

- information_schema.view_table_usage
- information_schema.partitions
