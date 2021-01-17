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
    use petgraph::dot::{Dot};
    let args = cli::new().get_matches();
    let src = args.value_of("SRC").unwrap_or("missing");
    // println!(
    //     "args: src={} dst={}",
    //     src,
    //     args.value_of("DEST").unwrap_or("missing")
    // );
    let mut conn = pg::connect(src);
    // let result = conn
    //     .query("select sha256, mode_bits from files limit 1", &[])
    //     .unwrap();
    // for row in result {
    //     println!("{:?}", row)
    // }
    let g = pg::get_fkey_dependency_graph(&mut conn);
    println!("{:?}", Dot::new(&g));
    // let result = pg::list_all_fkey_constraints(&mut conn);
    // for row in result {
    //     let constraint_name: &str = row.get("constraint_name");
    //     let table_name: &str = row.get("table_name");
    //     let column_name: &str = row.get("column_name");
    //     let foreign_table_name: &str = row.get("foreign_table_name");
    //     let foreign_column_name: &str = row.get("foreign_column_name");
    //     println!(
    //         "({}) --[col]-> ({}) --[{}]-> ({}) <-[col]-- ({})",
    //         table_name, column_name, constraint_name, foreign_column_name, foreign_table_name
    //     );
    // }
    // let result = pg::list_schemas(&mut conn);
    // for row in result {
    //     let name: &str = row.get("name");
    //     let owner: &str = row.get("owner");
    //     let access_privileges: &str = row.get("access_privileges");
    //     let desc: &str = row.get("description");
    //     println!(
    //         "schema::name={};\nowner={}\naccess_privileges={}\ndesc={}\n",
    //         name, owner, access_privileges, desc
    //     );
    // }
    // let result = pg::list_tables_in_schema(&mut conn, "public");
    // for row in result {
    //     let name: &str = row.get("name");
    //     let owner: &str = row.get("owner");
    //     let relkind: &str = row.get("relkind");
    //     println!("public.{}", name);
    // }
    // println!("schemas={:?}", result);
    // let result = conn.execute("select 1;", &[]).unwrap();
}
