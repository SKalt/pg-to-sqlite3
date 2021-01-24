use pg::{table_order, SchemaInformation};

// fn describe_tables() {}
// fn get_constraints() {}
// fn describe_views() {}
// fn apply_tables() {}
// fn apply_constraints() {}
// use crate::cli;
mod cli;
mod pg;
// mod petgraph;

fn main() {
    // use petgraph::dot::Dot;
    let args = cli::new().get_matches();
    let src = args.value_of("SRC").unwrap_or("missing");
    let mut conn = pg::connect(src);
    let sch = pg::SchemaInformation::new(&mut conn, "public");
    println!("{}", sch.dump_tables());
    // for (name, v) in &sch.views {
    //     println!("{}: {}", name, v.defn)
    // }
    let g = sch.to_dependency_graph();
    // table_order(&g);
    // println!("{:?}", Dot::new(&g));
}
