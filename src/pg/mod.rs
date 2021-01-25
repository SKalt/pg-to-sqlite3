// using Graph rather than petgraph::graphmap::DiGraphMap sicnce the latter
// doesn't allow parallel edges. The dependency graph within a databas schema
// might include multiple, parallel edges like "table A columng alpha depnds on
// (references) table B column beta; table A column x depends on table B column y"

use petgraph::graph::Graph;
use petgraph::{self, algo::toposort};
use postgres;
use postgres_types::Type as PgType;
use std::{collections::HashMap, fmt, u32, vec::Vec};
mod introspection;
mod object_types;
mod query;
mod validate;

use introspection::{
    get_all_fkey_constraints, get_table_defns, get_view_defns, get_view_refs,
    list_relations_in_schema,
};
use object_types::sqlite_type_from_pg_type;
pub use query::connect;

// TODO: constraint enum::{check, fkey, unique, pkey}
// TODO: implement rustqlite::ToSql as ToSqlite

#[derive(Debug, Clone)]
pub struct Table {
    oid: u32,
    name: String,
    columns: Vec<ColInfo>,
    approx_n_rows: i64,
}

fn create_sqlite_table_stmt(t: Table) -> String {
    let cols: Vec<String> = t.columns.iter().map(|col| format!("{}", col)).collect();
    return format!(
        "CREATE TABLE {} (\n  {}\n  );\n -- ~ {} rows\n\n",
        &t.name,
        cols.join("\n  , "),
        &t.approx_n_rows
    );
}

#[derive(Debug)]
pub struct FkeyConstraint {
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
pub struct View {
    pub oid: u32,
    pub name: String,
    pub defn: String,
    // materialized: bool?
}
#[derive(Debug)]
pub struct ViewRelUsage {
    view_oid: u32,
    view_name: String,
    rel_name: String,
    rel_oid: u32,
}

#[derive(Debug)]
pub struct SchemaInformation {
    pub name: String,
    // -- nodes --
    pub tables: HashMap<String, Table>,
    pub views: HashMap<String, View>,
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

pub struct Rel {
    oid: u32,
    name: String,
    relkind: String,
    approx_n_rows: i64,
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

pub fn table_order(g: &Graph<Node, Edge>) -> Vec<String> {
    // TODO: rename to "rel_order": not just tables
    let sorted = toposort(g, None);
    match sorted {
        Ok(mut r) => {
            r.reverse();
            return r.iter().map(|idx| (&(g[*idx]).name).to_owned()).collect();
        }
        Err(c) => panic!("{:?}", c),
    }
}

impl SchemaInformation {
    pub fn new(conn: &mut postgres::Client, schema: &str) -> SchemaInformation {
        let rels = list_relations_in_schema(conn, schema);
        let mut tables = HashMap::new();
        let mut views = HashMap::new();

        fn add_table(rel: Rel, tables: &mut HashMap<String, Table>) {
            let table = Table {
                oid: rel.oid,
                name: rel.name.to_owned(),
                approx_n_rows: rel.approx_n_rows,
                columns: vec![], // pupulated later
            };
            if tables.contains_key(&table.name) {
                panic!("duplicate table {}", table.name)
            }
            tables.insert(table.name.to_owned(), table);
        }
        fn add_view(rel: Rel, views: &mut HashMap<String, View>, tables: &HashMap<String, Table>) {
            let view = View {
                oid: rel.oid,
                name: rel.name.to_owned(),
                defn: "".to_owned(),
            };
            if views.contains_key(&view.name) {
                panic!("duplicate view {}", view.name);
            }
            if tables.contains_key(&view.name) {
                panic!("view {} conflicts with table {}", view.name, view.name);
            }
            views.insert(view.name.to_owned(), view);
        }

        for rel in rels {
            match rel.relkind.as_str() {
                "table" => add_table(rel, &mut tables),
                "view" => add_view(rel, &mut views, &tables),
                unknown => panic!("unrecognized rel type {}", unknown),
            }
        }

        get_table_defns(conn, &mut tables, schema);
        get_view_defns(conn, &mut views);

        let mut fkey_constraints = HashMap::new();
        for fk in get_all_fkey_constraints(conn, schema) {
            if fkey_constraints.contains_key(&fk.name) {
                panic!("duplicate foreign key name {}", fk.name); //
            }
            fkey_constraints.insert(fk.name.to_owned(), fk);
        }

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
        let duplicated = validate::validate_namespace(self);
        let missing = validate::validate_fkey_tables_present(self);
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
    pub fn table_order(&self) -> Vec<String> {
        let g = &self.to_dependency_graph();
        table_order(g)
    }
    pub fn dump_tables(&self) -> String {
        let tables: Vec<String> = self
            .table_order()
            .iter()
            .filter(|t| self.tables.contains_key(*t))
            .map(|t| {
                // let tbl: = self.tables.get(t);
                match self.tables.get(t) {
                    Some(x) => create_sqlite_table_stmt(x.clone()),
                    _ => panic!("unable to find table '{}' in {:?}", t, self.tables),
                }
            })
            .collect();
        return tables.join("\n");
    }
}

#[derive(Debug, Clone)]
pub struct ColInfo {
    name: String,
    data_type: PgType,
    nullable: bool,
}
impl fmt::Display for ColInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sqlite_col = sqlite_type_from_pg_type(&self.data_type)
            .unwrap()
            .to_string()
            .to_ascii_uppercase();
        write!(f, "{} {} --", self.name, sqlite_col)?;
        if self.nullable == false {
            write!(f, " NOT NULL")?;
        }
        write!(f, " -- {}", self.data_type)?;
        return Ok(());
    }
}
