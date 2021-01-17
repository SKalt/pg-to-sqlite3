// TODO: connect to a postgres database.
use petgraph::{self, graph::Node};
use postgres;
use std::vec::Vec;

// TODO: translate types from https://docs.rs/postgres/0.19.0/postgres/types/struct.Type.html

// TODO: define structs for postgres inspected
// - rel: can be a view
// - schema
// - table -> contraints
// - sequences
// - constraint::{check, fkey, unique, pkey}

// https://doxygen.postgresql.org/pg__dump_8c.html#a4ef904e6ea0a938a1fd5ec546dc920ac
// https://sigterm.sh/2010/07/09/generating-a-dependency-graph-for-a-postgresql-database/

// TODO: mock up a trait with the desired methods?
trait InspectedSchema {
    // name: &str
    // oid?
    // tables() -> Vec<InspectedTable>
    // views() -> Vec<InspectedView>
    // constraints()?
    // sequences()?
}

// https://docs.rs/petgraph/0.5.1/petgraph/algo/fn.toposort.html

// information_schema tables of note:

// information_schema.check_constraints
// information_schema.columns
// information_schema.key_column_usage
// information_schema.referential_contraints
// information_schema.schemata
// information_schema.table_constraints
// information_schema.table_privileges
// information_schema.views

// less portable, but still interesting:

// information_schema.view_table_usage
// information_schema.partitions

trait InspectedRel {
    // relkind: &str
}

pub fn connect(connection_string: &str) -> postgres::Client {
    // let result = ;
    match postgres::Client::connect(connection_string, postgres::NoTls) {
        Ok(conn) => return conn,
        Err(e) => panic!("{:?}", e),
    }
}

fn must_succeed(response: Result<Vec<postgres::Row>, postgres::Error>) -> Vec<postgres::Row> {
    match response {
        Ok(rows) => return rows,
        Err(e) => panic!("{:?}", e),
    }
}

pub fn pretty_relkind(relkind: &str) -> &str {
    match relkind {
        "r" => return "table",
        "v" => return "view",
        "m" => return "materialized view",
        "i" => return "index",
        "S" => return "sequence",
        "s" => return "special",
        "f" => return "foreign table",
        "p" => return "partitioned view",
        "I" => return "partitioned index",
        other => panic!("unexpected relkind {:?}", other),
    }
}

pub fn list_schemas(conn: &mut postgres::Client) -> Vec<postgres::Row> {
    return must_succeed(conn.query(
        "
        SELECT
            n.oid,
            n.nspname AS name,
            pg_catalog.pg_get_userbyid(n.nspowner) AS Owner,
            pg_catalog.array_to_string(n.nspacl, E'\n') AS access_privileges,
            pg_catalog.obj_description(n.oid, 'pg_namespace') AS description
        FROM pg_catalog.pg_namespace n
        WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
        ORDER BY 2;
        ",
        &[],
    ));
}

pub fn list_tables_in_schema(conn: &mut postgres::Client, schema_name: &str) -> Vec<postgres::Row> {
    return must_succeed(conn.query(
        "
        SELECT
            c.oid,
            c.relname AS name,
            c.relkind,
            pg_catalog.pg_get_userbyid(c.relowner) as owner
        FROM pg_catalog.pg_class c
            LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind IN ('r','p','s','')
            AND n.nspname = $1
            AND pg_catalog.pg_table_is_visible(c.oid)
        ORDER BY 1, 2;
        ",
        &[&schema_name],
    ));
}

pub fn list_views_in_schema(conn: &mut postgres::Client, schema: &str) -> Vec<postgres::Row> {
    return must_succeed(conn.query(
        "
        SELECT
            n.nspname AS schema,
            c.relname AS name,
            CASE c.relkind,
            pg_catalog.pg_get_userbyid(c.relowner) AS owner
        FROM pg_catalog.pg_class c
            LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind IN ('v','')
            AND n.nspname <> 'pg_catalog'
            AND n.nspname <> 'information_schema'
            AND n.nspname = $1
            AND pg_catalog.pg_table_is_visible(c.oid)
        ORDER BY 1,2;",
        &[&schema],
    )); // might overlap w/ tables?
}

pub fn list_all_fkey_constraints(conn: &mut postgres::Client) -> Vec<postgres::Row> {
    return must_succeed(conn.query(
        "
        SELECT
            table_contraints.constraint_name,
            table_contraints.table_name,
            key_column_usage.column_name,
            constraint_column_usage.table_name AS foreign_table_name,
            constraint_column_usage.column_name AS foreign_column_name
        FROM information_schema.table_constraints AS table_contraints
        JOIN
            information_schema.key_column_usage AS key_column_usage ON
            table_contraints.constraint_name = key_column_usage.constraint_name
        JOIN information_schema.constraint_column_usage AS constraint_column_usage ON
            constraint_column_usage.constraint_name = table_contraints.constraint_name
        WHERE constraint_type = 'FOREIGN KEY'
        ",
        &[],
    ));
}
// TODO: define node types, edge types
pub fn get_fkey_dependency_graph(conn: &mut postgres::Client) -> petgraph::Graph<String, String> {
    use std::collections::HashMap;
    use petgraph::Graph;
    use petgraph::graph::NodeIndex;

    let mut deps = Graph::<String, String>::new();
    let mut names = HashMap::<String, NodeIndex>::new();
    let fkey_rows = list_all_fkey_constraints(conn);

    fn ensure_node<'a, 'b>(
        name: String,
        names: &'b mut HashMap::<String, NodeIndex>,
        deps: &'b mut Graph::<String, String>
    ) -> &'b mut NodeIndex {
        let n = name.to_owned();
        let insert = || deps.add_node(n);
        let node: &mut NodeIndex = names.entry(name).or_insert_with(insert);
        return node;
    }
    
    for row in fkey_rows {
        let src_name: &str = row.get("table_name");
        let target_name: &str = row.get("foreign_table_name");
        let constraint_name: &str = row.get("constraint_name");
        let src: NodeIndex = *ensure_node(src_name.to_owned(), &mut names, &mut deps);
        let target: NodeIndex = *ensure_node(target_name.to_owned(), &mut names, &mut deps);
        deps.add_edge(src, target, constraint_name.to_owned());
    }

    return deps;
}

pub fn list_table_constraints(/* schema, table */) /* -> constraints */ {}
pub fn dump_data_from_table(/* schema, table */) /* something streaming */ {}
pub fn count_rows_in_table(/* schema, table */) /* nubmber */ {}
