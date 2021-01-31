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

pub fn create_all_views(conn: &mut Connection, view_defns: &str) -> Result<(), Error> {
    let result = conn.execute_batch(view_defns);
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", view_defns);
            Err(e)
        }
    }
}
