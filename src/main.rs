mod cli;
mod pg;
use cli::new;
use fallible_iterator::FallibleIterator;
use rusqlite::Connection;
use std::iter;
fn main() {
    // use petgraph::dot::Dot;
    let args = cli::new().get_matches();
    let src = args.value_of("SRC").unwrap(); // enforced by clap
    let dest = args.value_of("DEST").unwrap();
    // TODO: validate the dest's parent directory exists
    // TODO: if the dest _file_ exists, require an --overwrite arg
    let mut conn = pg::connect(src);
    let sch = pg::SchemaInformation::new(&mut conn, "public");
    println!("{}", sch.dump_tables());
    pg::do_the_thing(dest, &sch.dump_tables()).unwrap();
    // let mut rows = pg::dump_table(&mut conn, "_file").unwrap();
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
