// rustqlite goes here!
// fn dump_table_data(row_defn, rows) {}
// fn copy_view(v) {}
// reinstate indices
// fn reinstate_constraint(c) {}
use rusqlite::{Connection, Error};

pub fn create_all_tables(conn: &mut Connection, create_table_stmt: &str) -> Result<(), Error> {
    let txn = conn.transaction()?;
    txn.execute_batch(create_table_stmt)?;
    let result = txn.commit();
    return result;
}
// TODO: create tables
// TODO: dump tables
