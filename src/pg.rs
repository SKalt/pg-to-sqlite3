// using Graph rather than petgraph::graphmap::DiGraphMap sicnce the latter
// doesn't allow parallel edges. The dependency graph within a databas schema
// might include multiple, parallel edges like "table A columng alpha depnds on
// (references) table B column beta; table A column x depends on table B column y"

use petgraph::graph::{Graph, NodeIndex};
use petgraph::{self, algo::toposort};
use postgres;
use std::{
    collections::{HashMap, HashSet},
    u32,
    vec::Vec,
};

// - sequences
// TODO: constraint enum::{check, fkey, unique, pkey}
// TODO: implement rustqlite::ToSql as ToSqlite

#[derive(Debug)]
struct Table {
    oid: u32,
    name: String,
    columns: Vec<postgres::Column>,
}

#[derive(Debug)]
struct FkeyConstraint {
    // oid: u32,
    /// the name of the constraint
    name: String,
    table: String,
    column: String,
    foreign_table: String,
    foreign_column: String,
}
#[derive(Debug)]
struct CheckConstraint {
    oid: u32,
    defn: String,
}
#[derive(Debug)]
struct View {
    oid: u32,
    name: String,
    defn: String,
    // materialized: bool?
}
#[derive(Debug)]
struct ViewRelUsage {
    view_oid: u32,
    view_name: String,
    rel_name: String,
    rel_oid: u32,
}

#[derive(Debug)]
pub struct SchemaInformation {
    name: String,
    // -- nodes --
    tables: HashMap<String, Table>, // columns may/not be unpopulated
    views: HashMap<String, View>,
    // check_constraints,
    // not_null_constraints,
    // -- edges --
    fkey_constraints: HashMap<String, FkeyConstraint>,
    view_rel_usage: Vec<ViewRelUsage>, // TODO: HashMap<String, ViewTableUsage>
                                       // -- ??? ---
                                       // extensions (specifically, for translating postGIS -> spatialite)
                                       // relations, // the namespace tables+views inhabit ?
                                       // sequences  // this one's going to be difficult to replicate, due to sqlite's `rowid` trick
                                       // see https://www.sqlitetutorial.net/sqlite-autoincrement/
}

fn validate_namespace(si: &SchemaInformation) -> Result<(), String> {
    let mut namespace = HashSet::new();
    // assert all tables are unique
    let mut duplicates: Vec<String> = vec![];
    for (name, _) in &si.tables {
        let present = namespace.insert(name);
        if present {
            duplicates.push(format!("duplicate table {}", name));
        }
    }
    for (name, _) in &si.views {
        let present = namespace.insert(name);
        if present {
            duplicates.push(format!("duplicate view {}", name));
        }
    }
    if duplicates.len() > 0 {
        return Err(format!(
            "{} relations have duplicate names:  \n{}",
            duplicates.len(),
            duplicates.join("\n  ")
        ));
    } else {
        return Ok(());
    }
}

fn validate_fkey_tables_present(si: &SchemaInformation) -> Result<(), String> {
    let mut missing = vec![];
    for (_, fk) in &si.fkey_constraints {
        if !si.tables.contains_key(&fk.table) {
            missing.push(format!("missing table {} from {}", fk.table, fk.name));
        }
        if !si.tables.contains_key(&fk.foreign_table) {
            missing.push(format!(
                "missing foreign table {} from {}",
                fk.foreign_table, fk.name
            ));
        }
    }
    if missing.len() == 0 {
        return Ok(());
    } else {
        return Err(format!(
            "Some tables referenced by constraints were missing:  \n{}",
            missing.join("\n  ")
        ));
    }
}

struct Rel {
    oid: u32,
    name: String,
    relkind: String,
}

#[derive(Debug, Clone)]
pub struct Node {
    name: String,
    type_: String,
}
#[derive(Debug)]
pub struct Edge {
    type_: String,
}

pub fn table_order(g: &Graph<Node, Edge>) {
    let sorted = toposort(g, None);
    match sorted {
        Ok(mut r) => {
            r.reverse();
            for idx in r {
                let q: &str = &(g[idx]).name;
                println!("{:?}", q);
            }
        }
        Err(c) => panic!("{:?}", c),
    }
}

impl SchemaInformation {
    pub fn new(conn: &mut postgres::Client, schema: &str) -> SchemaInformation {
        let rels = list_relations_in_schema(conn, schema);

        // lookup tables
        let mut tables = HashMap::new();

        let _tables = rels
            .iter()
            .filter(|rel| rel.relkind == "table")
            .map(|rel| Table {
                oid: rel.oid,
                name: rel.name.to_owned(),
                columns: vec![],
            });
        for table in _tables {
            if tables.contains_key(&table.name) {
                panic!("duplicate table {}", table.name)
            }
            tables.insert(table.name.to_owned(), table);
        }
        // TODO: lookup table columns
        // lookup views
        let mut views = HashMap::new();
        let _views = rels
            .iter()
            .filter(|rel| rel.relkind == "view")
            .map(|rel| View {
                oid: rel.oid,
                name: rel.name.to_owned(),
                defn: "".to_owned(),
            });
        for view in _views {
            if views.contains_key(&view.name) {
                panic!("duplicate view {}", view.name);
            }
            if tables.contains_key(&view.name) {
                panic!("view {} conflicts with table {}", view.name, view.name);
            }
            views.insert(view.name.to_owned(), view);
        }
        let mut fkey_constraints = HashMap::new();
        for fk in get_all_fkey_constraints(conn, schema) {
            if fkey_constraints.contains_key(&fk.name) {
                panic!("duplicate foreign key name {}", fk.name); //
            }
            fkey_constraints.insert(fk.name.to_owned(), fk);
        }

        // TODO: look up view definitions
        let view_rel_usage = get_view_refs(conn, schema);
        return SchemaInformation {
            name: schema.to_owned(),
            tables,
            views,
            fkey_constraints,
            view_rel_usage,
        };
    }
    fn validate(&self) -> Result<(), String> {
        // ValidationError implementation
        let mut all_errors = vec![];
        let duplicated = validate_namespace(self);
        let missing = validate_fkey_tables_present(self);
        match duplicated {
            Ok(_) => {}
            Err(v) => all_errors.push(v),
        };
        match missing {
            Ok(_) => {}
            Err(v) => all_errors.push(v),
        };
        if !all_errors.is_empty() {
            let errs: Vec<String> = all_errors.iter().map(|e| e.to_string()).collect();
            return Err(errs.join("\n  "));
        } else {
            return Ok(());
        }
        //     let namespace = HashSet::new();
        //     let errors: Vec<&str> = vec![];
        //     assert all table, view names distinct
        //     assert all fkey constraints' tables & foreign tables are present
    }
    pub fn to_dependency_graph(&self) -> Graph<Node, Edge> {
        let mut names = HashMap::new();
        let mut deps = Graph::new();

        for (name, _) in &self.tables {
            let n = deps.add_node(Node {
                name: name.to_owned(),
                type_: "t".to_owned(),
            });
            names.insert(name, n);
        }
        for (name, _) in &self.views {
            let n = deps.add_node(Node {
                name: name.to_owned(),
                type_: "v".to_owned(),
            });
            names.insert(name, n);
        }

        // println!()
        for usage in &self.view_rel_usage {
            let table = names.get(&usage.rel_name).unwrap();
            let view = names.get(&usage.view_name).unwrap();
            deps.add_edge(
                *table,
                *view,
                Edge {
                    type_: "v->r".to_owned(),
                },
            );
        }
        for (_, fk) in &self.fkey_constraints {
            let src = names.get(&fk.table).unwrap();
            let dest = names.get(&fk.foreign_table).unwrap();
            deps.add_edge(
                *src,
                *dest,
                Edge {
                    type_: "fk".to_owned(),
                },
            );
        }
        return deps;
    }
    pub fn table_order(&self) {
        let g = &self.to_dependency_graph();
        table_order(g)
    }
}

pub fn connect(connection_string: &str) -> postgres::Client {
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

pub fn _pretty_relkind(relkind: &str) -> &str {
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
fn pretty_deptype(deptype: &str) -> &str {
    match deptype {
        "n" => return "normal",
        "a" => return "automatic",
        "i" => return "internal",
        "e" => return "extension",
        "p" => return "pinned",
        other => panic!("unknown deptype '{:?}'", other),
    }
}
pub fn list_schemas(conn: &mut postgres::Client) -> Vec<String> {
    // TODO: deprecate? We only need to check 1 schema.
    return must_succeed(conn.query(
        "
        SELECT schema_name
        FROM information_schema.schemata
        WHERE schema_name NOT LIKE 'pg_%' AND schema_name != 'information_schema';
        ",
        &[],
    ))
    .iter()
    .map(|row| row.get("schema_name"))
    .collect();
}

fn list_relations_in_schema(conn: &mut postgres::Client, schema_name: &str) -> Vec<Rel> {
    return must_succeed(conn.query(
        "
        SELECT
            c.oid,
            c.relname AS name,
            c.relkind::TEXT,
            pg_catalog.pg_get_userbyid(c.relowner) as owner
        FROM pg_catalog.pg_class c
            LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind IN ('r','v','m','s','p')
            AND n.nspname = $1
            AND pg_catalog.pg_table_is_visible(c.oid)
        ORDER BY 1, 2;
        ",
        &[&schema_name],
    ))
    .iter()
    .map(|row| {
        let oid = row.get("oid");
        let name = row.get("name");
        let relkind = _pretty_relkind(row.get("relkind")).to_owned();
        return Rel { oid, name, relkind };
    })
    .collect();
}

// TODO: parametrize with a Vec<str> schema names
pub fn list_all_fkey_constraints(conn: &mut postgres::Client, schema: &str) -> Vec<postgres::Row> {
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
            AND table_contraints.constraint_schema = $1
        ",
        &[&schema],
    ));
}

fn get_all_fkey_constraints(conn: &mut postgres::Client, schema: &str) -> Vec<FkeyConstraint> {
    return list_all_fkey_constraints(conn, schema)
        .iter()
        .map(|row| {
            let table = row.get("table_name");
            // let table = get_name("table_name")(row);
            let col = row.get("column_name");
            let constraint = row.get("constraint_name");
            let foreign_table = row.get("foreign_table_name");
            let foreign_column = row.get("foreign_column_name");
            return FkeyConstraint {
                table,
                column: col,
                name: constraint,
                foreign_table,
                foreign_column,
            };
        })
        .collect();
}

// TODO: define node types, edge types
pub fn get_fkey_dependency_graph(
    conn: &mut postgres::Client,
    schema: &str,
) -> petgraph::Graph<String, String> {
    let mut deps = Graph::<String, String>::new();
    let mut names = HashMap::<String, NodeIndex>::new();
    let fkeys = get_all_fkey_constraints(conn, schema);
    fn ensure_node<'a, 'b>(
        name: String,
        names: &'b mut HashMap<String, NodeIndex>,
        deps: &'b mut Graph<String, String>,
    ) -> &'b mut NodeIndex {
        let n = name.to_owned();
        let insert = || deps.add_node(n);
        let node: &mut NodeIndex = names.entry(name).or_insert_with(insert);
        return node;
    }

    for fkey in fkeys {
        let src: NodeIndex = *ensure_node(fkey.table, &mut names, &mut deps);
        let target: NodeIndex = *ensure_node(fkey.foreign_table, &mut names, &mut deps);
        deps.add_edge(src, target, fkey.name.to_owned());
    }

    return deps;
}

fn list_view_dependencies(conn: &mut postgres::Client, schema: &str) -> Vec<ViewRelUsage> {
    return must_succeed(conn.query(
        "
        SELECT DISTINCT
            source_rel.oid AS source_oid,
            source_rel.relname AS source_table,
            dependent_rel.relname AS dependent_rel,
            dependent_rel.oid AS dependent_oid
        FROM pg_catalog.pg_depend AS dep
        JOIN pg_catalog.pg_rewrite AS rewrite ON dep.objid = rewrite.oid
        JOIN pg_catalog.pg_class AS dependent_rel ON rewrite.ev_class = dependent_rel.oid
        JOIN pg_catalog.pg_class AS source_rel ON dep.refobjid = source_rel.oid
        JOIN pg_catalog.pg_namespace source_ns ON source_ns.oid = source_rel.relnamespace
        WHERE source_ns.nspname = $1
            AND source_rel.oid <> dependent_rel.oid
        ",
        &[&schema],
    ))
    .iter()
    .map(|row| {
        let view_oid: u32 = row.get("src_oid");
        let table_oid: u32 = row.get("dest_oid");
        let view_name = row.get("view_name");
        let table_name = row.get("table_name");
        return ViewRelUsage {
            view_oid,
            view_name,
            rel_name: table_name,
            rel_oid: table_oid,
        };
    })
    .collect();
}

fn get_view_refs(conn: &mut postgres::Client, schema: &str) -> Vec<ViewRelUsage> {
    return must_succeed(conn.query(
        "
        SELECT DISTINCT
            source_rel.oid        AS source_oid,
            source_rel.relname    AS source_table,
            dependent_rel.relname AS dependent_rel,
            dependent_rel.oid     AS dependent_oid
        FROM pg_catalog.pg_depend AS dep
        JOIN pg_catalog.pg_rewrite AS rewrite ON dep.objid = rewrite.oid
        JOIN pg_catalog.pg_class AS dependent_rel ON rewrite.ev_class = dependent_rel.oid
        JOIN pg_catalog.pg_class AS source_rel ON dep.refobjid = source_rel.oid
        JOIN pg_catalog.pg_namespace source_ns ON source_ns.oid = source_rel.relnamespace
        WHERE source_ns.nspname = $1 AND source_rel.oid <> dependent_rel.oid
        ",
        &[&schema], // TODO: parametrize schema
    ))
    .iter()
    .map(|row| {
        let view_oid: u32 = row.get("source_oid");
        let view_name = row.get("source_table");
        let rel_name = row.get("dependent_rel");
        let rel_oid: u32 = row.get("dependent_oid");
        return ViewRelUsage {
            view_oid,
            view_name,
            rel_name,
            rel_oid,
        };
    })
    .collect();
}

pub fn list_table_constraints(/* schema, table */) /* -> constraints */ {}
pub fn dump_data_from_table(/* schema, table */) /* something streaming */ {}
pub fn count_rows_in_table(/* schema, table */) /* nubmber */ {}
