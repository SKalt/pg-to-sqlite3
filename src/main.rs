mod cli;
mod pg;

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
    // for (name, v) in &sch.views {
    //     println!("{}: {}", name, v.defn)
    // }
    // let g = sch.to_dependency_graph();
    // table_order(&g);
    // println!("{:?}", Dot::new(&g));
}
