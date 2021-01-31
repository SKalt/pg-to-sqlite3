mod cli;
mod pg;
mod sqlite;
use cli::new;
use core::panic;
use fallible_iterator::FallibleIterator;
use pg::transfer_table_rows;
use rusqlite::Connection;
use std::iter;
fn main() -> Result<(), pg::SqlError> {
    // use petgraph::dot::Dot;
    let args = cli::new().get_matches();
    let src = args.value_of("SRC").unwrap(); // enforced by clap
    let dest = args.value_of("DEST").unwrap();
    let no_views = args.is_present("no_views");
    let schema_name: &str = args.value_of("schema").unwrap();
    // TODO: validate the dest's parent directory exists
    // TODO: if the dest _file_ exists, require an --overwrite arg
    let mut conn = pg::connect(src);
    let sch = pg::SchemaInformation::new(&mut conn, schema_name);
    let mut lite = rusqlite::Connection::open(dest).unwrap();
    sqlite::create_all_tables(&mut lite, &sch.create_table_statements())?;

    if no_views {
        println!("skipping view creation");
    } else {
        sqlite::create_all_views(&mut lite, &sch.create_view_statements())?;
    }

    let mut txn = lite.transaction()?;
    for table_name in sch.order {
        match &sch.tables.get(&table_name) {
            Some(tbl) => {
                println!("transferring {}", &table_name);
                pg::transfer_table_rows(&mut conn, &mut txn, tbl)?;
            }
            _ => {} // not a table
        }
    }
    println!("committing rows...");
    txn.commit()?;
    println!("committed.");
    // now indices & fkey costraints
    Ok(())

    // println!("{:?}", Dot::new(&g));
}
