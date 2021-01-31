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
    // TODO: validate the dest's parent directory exists
    // TODO: if the dest _file_ exists, require an --overwrite arg
    let mut conn = pg::connect(src);
    let sch = pg::SchemaInformation::new(&mut conn, "public");
    let mut lite = rusqlite::Connection::open(dest).unwrap();
    sqlite::create_all_tables(&mut lite, &sch.create_table_statements())?;
    let mut txn = lite.transaction()?;
    for table_name in sch.table_order {
        println!("transferring {}", &table_name);
        let tbl = &sch.tables.get(&table_name).unwrap();
        pg::transfer_table_rows(&mut conn, &mut txn, tbl)?;
    }
    println!("committing...");
    txn.commit()?;
    println!("committed.");
    Ok(())
    // for tbl in sch
    //     .get_table_order()
    //     .iter()
    //     .map(|table_name| &sch.tables.get(table_name).unwrap())
    // {
    //     pg::transfer_table_rows(&mut conn, &mut lite, tbl)?;
    // }

    // while let Some(row) = rows.next().unwrap() {
    //     println!("{:?}", row);
    // }
    // for (name, v) in &sch.views {
    //     println!("{}: {}", name, v.defn)
    // }
    // let g = sch.to_dependency_graph();
    // table_order(&g);
    // println!("{:?}", Dot::new(&g));
}
