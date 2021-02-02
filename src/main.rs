mod cli;
mod pg;
mod sqlite;
use cli::new;
use core::panic;
use fallible_iterator::FallibleIterator;
use pg::transfer_table_rows;
use rusqlite::Connection;
use std::{fs, path::Path};

fn main() -> Result<(), pg::SqlError> {
    let args = cli::new().get_matches();

    let src = args.value_of("SRC").unwrap(); // enforced by clap
    let dest = args.value_of("DEST").unwrap();
    let schema_name: &str = args.value_of("schema").unwrap();
    let overwrite = args.is_present("overwrite");
    let no_views = args.is_present("no_views");
    let schema_only = args.is_present("schema_only");
    let data_only = args.is_present("data_only");

    let mut conn = pg::connect(src);
    let sch = pg::SchemaInformation::new(&mut conn, schema_name);

    if dest == "stdout" || dest == "STDOUT" {
        if data_only {
            println!("-- skipping table creation");
        } else {
            println!("{}", &sch.create_table_statements());
        }
        if no_views || data_only {
            println!("-- skipping view creation");
        } else {
            println!("{}", &sch.create_view_statements());
        }
        return Ok(());
    } else {
        let dest_file = Path::new(dest);
        if dest_file.exists() {
            if !dest_file.is_file() {
                panic!("{} is not a file", dest);
            } else {
                let dest_metadata = fs::metadata(dest_file).unwrap();
                if !overwrite {
                    assert_eq!(
                        dest_metadata.len(),
                        0,
                        "{} is already populated; pass `--overwrite` if you'd like to overwrite it",
                        dest
                    );
                }
            }
        }
    }
    // TODO: if the dest _file_ exists, require an --overwrite arg
    let mut lite = rusqlite::Connection::open(dest).unwrap();

    if data_only {
        println!("-- skipping table creation");
    } else {
        sqlite::create_all_tables(&mut lite, &sch.create_table_statements())?;
    }

    if no_views || data_only {
        println!("-- skipping view creation");
    } else {
        sqlite::create_all_views(&mut lite, &sch.create_view_statements())?;
    }

    if schema_only {
        println!("-- skipping data insertion");
    } else {
        lite.set_db_config(
            rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY,
            false,
        )?;
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
        lite.set_db_config(
            rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY,
            true,
        )?;
    }

    // now indices
    Ok(())

    // use petgraph::dot::Dot;
    // println!("{:?}", Dot::new(&g));
}
