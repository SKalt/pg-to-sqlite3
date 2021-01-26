// rustqlite goes here!
// fn dump_table_data(row_defn, rows) {}
// fn copy_view(v) {}
// fn reinstate_constraint(c) {}
use rusqlite::{params, Connection, Result};

pub fn connect(path: &str) -> Result<Connection> {
    return Connection::open(path);
}

fn create_all_tables(conn: &mut Connection, create_table_stmt: &str) {
    conn
}
// TODO: create tables
// TODO: dump tables
