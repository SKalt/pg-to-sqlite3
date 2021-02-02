// using Graph rather than petgraph::graphmap::DiGraphMap sicnce the latter
// doesn't allow parallel edges. The dependency graph within a databas schema
// might include multiple, parallel edges like "table A columng alpha depnds on
// (references) table B column beta; table A column x depends on table B column y"

use fmt::Formatter;
use petgraph::graph::Graph;
use petgraph::{self, algo::toposort};
use postgres::{self, RowIter, Transaction};
use postgres_types::Type as PgType;
use std::{
    collections::{HashMap, VecDeque},
    convert::TryInto,
    fmt,
    intrinsics::transmute,
    u32,
    vec::Vec,
};
mod introspection;
mod object_types;
mod query;
mod validate;
use fallible_iterator::FallibleIterator;

use introspection::{
    get_all_fkey_constraints, get_all_pkey_constraints, get_all_unique_constraints,
    get_table_defns, get_view_defns, get_view_refs, list_relations_in_schema,
};
use object_types::{sqlite_type_from_pg_type, translate_row};
pub use query::connect;

// TODO: constraint enum::{check, fkey, unique, pkey}
// TODO: implement rustqlite::ToSql as ToSqlite

#[derive(Debug, Clone)]
pub struct Table {
    oid: u32,
    name: String,
    column_order: Vec<String>,
    columns: HashMap<String, ColInfo>,
    pk_constraints: Vec<PkeyConstraint>,
    fkey_constraints: Vec<FkeyConstraint>,
    unique_constraints: Vec<UniqueConstraint>,
    approx_n_rows: i64,
}

fn create_sqlite_table_stmt(t: Table) -> String {
    return format!("{}", t);
    // let cols: Vec<String> = t
    //     .column_order
    //     .iter()
    //     .map(|col_name| t.columns.get(col_name).unwrap())
    //     .map(|col| format!("{}", col))
    //     .collect();
    // return format!(
    //     "CREATE TABLE {} (\n  {}\n{});\n  -- ~ {} rows\n",
    //     &t.name,
    //     cols.join("\n  , "),
    //     &t.fkey_constraints
    //         .iter()
    //         .map(|fk| format!("{}", fk))
    //         .collect::<Vec<String>>()
    //         .join("\n  ,"),
    //     &t.approx_n_rows
    // );
}

#[derive(Debug, Clone)]
pub struct PkeyConstraint {
    name: String,
    table: String,
    columns: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct UniqueConstraint {
    name: String,
    table: String,
    columns: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct FkeyConstraint {
    name: String,
    table: String,
    columns: Vec<String>,
    foreign_table: String,        // could be Vec<String>
    foreign_columns: Vec<String>, // could be Vec<String>
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
    pub order: Vec<String>,
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
    dependency_graph: Graph<Node, Edge>,
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

pub fn rel_order(g: &Graph<Node, Edge>) -> Vec<String> {
    // TODO: rename to "rel_order": not just tables
    let sorted = toposort(g, None);
    match sorted {
        Ok(mut r) => {
            r.reverse();
            return r
                .iter()
                // .filter(|idx| (&(g[**idx]).type_ == "t"))
                .map(|idx| (&(g[*idx]).name).to_owned())
                .collect();
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
                column_order: vec![],
                fkey_constraints: vec![],
                unique_constraints: vec![],
                pk_constraints: vec![],
                columns: HashMap::new(), // pupulated later
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
            let tbl = tables.get_mut(&fk.table).unwrap();
            tbl.fkey_constraints.push(fk.clone());
            fkey_constraints.insert(fk.name.to_owned(), fk);
        }
        for pk in get_all_pkey_constraints(conn, schema) {
            let tbl = tables.get_mut(&pk.table).unwrap();
            tbl.pk_constraints.push(pk);
            // TODO: validate pk name uniqueness?
        }
        for uq in get_all_unique_constraints(conn, schema) {
            let tbl = tables.get_mut(&uq.table).unwrap();
            tbl.unique_constraints.push(uq);
        }
        let view_rel_usage = get_view_refs(conn, schema);
        let dependency_graph =
            to_dependency_graph(&tables, &views, &view_rel_usage, &fkey_constraints);
        let table_order = rel_order(&dependency_graph);
        to_dependency_graph(&tables, &views, &view_rel_usage, &fkey_constraints);

        return SchemaInformation {
            name: schema.to_owned(),
            tables,
            views,
            fkey_constraints,
            view_rel_usage,
            dependency_graph,
            order: table_order, // this gets filled in later
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

    pub fn create_table_statements(&self) -> String {
        let tables: Vec<String> = self
            .order
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
    pub fn create_view_statements(&self) -> String {
        let views: Vec<String> = self
            .order
            .iter()
            .filter(|name| self.views.contains_key(*name))
            .map(|name| {
                let view = self.views.get(name).unwrap();
                return format!("CREATE VIEW {} AS\n{}\n", view.name, view.defn);
            })
            .collect();
        return views.join("\n");
    }
}

pub fn to_dependency_graph(
    tables: &HashMap<String, Table>,
    views: &HashMap<String, View>,
    view_rel_usage: &Vec<ViewRelUsage>,
    fkey_constraints: &HashMap<String, FkeyConstraint>,
) -> Graph<Node, Edge> {
    let mut names = HashMap::new();
    let mut deps = Graph::new();

    for (name, _) in tables {
        let n = deps.add_node(Node {
            name: name.to_owned(),
            type_: "t".to_owned(),
        });
        names.insert(name, n);
    }
    for (name, _) in views {
        let n = deps.add_node(Node {
            name: name.to_owned(),
            type_: "v".to_owned(),
        });
        names.insert(name, n);
    }

    // println!()
    for usage in view_rel_usage {
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
    for (_, fk) in fkey_constraints {
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
#[derive(Debug, Clone)]
pub struct ColInfo {
    name: String,
    data_type: PgType,
    nullable: bool,
}

impl fmt::Display for ColInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sqlite_type = sqlite_type_from_pg_type(&self.data_type)
            .unwrap()
            .to_string()
            .to_ascii_uppercase();

        write!(
            f,
            "{} {} -- {}",
            self.name,
            sqlite_type,
            self.data_type.to_string().to_ascii_uppercase()
        )?;
        if self.nullable == false {
            write!(f, " NOT NULL")?;
        }
        return Ok(());
    }
}

impl fmt::Display for PkeyConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CONSTRAINT {} PRIMARY KEY ({})",
            self.name,
            self.columns.join(", ")
        )
    }
}

impl fmt::Display for UniqueConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CONSTRAINT {} UNIQUE ({})",
            self.name,
            self.columns.join(", ")
        )
    }
}
impl fmt::Display for FkeyConstraint {
    /// https://www.sqlite.org/syntax/table-constraint.html
    /// https://www.sqlite.org/syntax/foreign-key-clause.html
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}({})",
            self.name,
            self.columns.join(", "),
            self.table,
            self.foreign_columns.join(", ")
        )
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cols: Vec<String> = self
            .column_order
            .iter()
            .map(|col_name| self.columns.get(col_name).unwrap())
            .map(|col| format!("{}", col))
            .collect();
        write!(
            f,
            "CREATE TABLE {} (\n  {}\n",
            &self.name,
            cols.join("\n  , "),
        )?;
        let fk_constraints = self
            .fkey_constraints
            .iter()
            .map(|fk| format!("{}", fk))
            .collect::<Vec<String>>();
        let pk_constraints = self
            .pk_constraints
            .iter()
            .map(|pk| format!("{}", pk))
            .collect::<Vec<String>>();
        let unique_constraints = self
            .unique_constraints
            .iter()
            .map(|pk| format!("{}", pk))
            .collect::<Vec<String>>();
        let constraints = [pk_constraints, unique_constraints, fk_constraints].concat();
        if self.fkey_constraints.len() > 0 {
            write!(f, "  , {}\n", constraints.join("\n  , "))?;
        }
        write!(f, "); -- ~ {} rows\n", self.approx_n_rows)?;
        Ok(())
    }
}

pub fn dump_table<'a, 'b>(
    conn: &'a mut postgres::Client,
    table: &'b str,
) -> Result<postgres::RowIter<'a>, postgres::Error> {
    let query = format!("select * from {}", table);
    let statement: postgres::Statement = conn.prepare(&query)?;
    let params: Vec<&str> = vec![];
    return conn.query_raw(&statement, params.iter());
}

use postgres::Error as PgError;
use rusqlite::{Connection, Error as SqliteErr, Transaction as SqliteTransaction};

#[derive(Debug)]
pub enum SqlError {
    SqliteErr(SqliteErr),
    PgError(PgError),
}

impl From<SqliteErr> for SqlError {
    fn from(e: SqliteErr) -> Self {
        return SqlError::SqliteErr(e);
    }
}
impl From<PgError> for SqlError {
    fn from(e: PgError) -> Self {
        return SqlError::PgError(e);
    }
}

pub fn transfer_table_rows(
    pg: &mut postgres::Client,
    lite: &mut SqliteTransaction,
    table: &Table,
) -> Result<(), SqlError> {
    let mut rows = dump_table(pg, &table.name)?;
    let col_params: Vec<String> = table.columns.iter().map(|_| "?".to_owned()).collect();
    let insert = format!(
        "INSERT INTO {} VALUES ({})",
        &table.name,
        col_params.join(", ")
    );
    // pg and sqlite tables _MUST_ have the same name and column order
    let statement = &mut lite.prepare(&*insert)?; // causes stack overflow?
    let countdown: u64 = (table.approx_n_rows).try_into().unwrap(); // safe since we don't expect negative numbers of rows
    let pb = indicatif::ProgressBar::new(countdown);

    while let Some(row) = rows.next()? {
        statement.execute(translate_row(&row))?;
        pb.inc(1);
    }
    Ok(())
}
