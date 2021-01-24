pub fn get_fkey_dependency_graph(
    conn: &mut postgres::Client,
    schema: &str,
) -> petgraph::Graph<String, String> {
    let mut deps = Graph::<String, String>::new();
    let mut names = HashMap::<String, NodeIndex>::new();
    let fkeys = get_all_fkey_constraints(conn, schema);
    fn ensure_node<'a, 'b>(
        name: String,
        names: &'b mut HashMap<String, NodeIndex>,
        deps: &'b mut Graph<String, String>,
    ) -> &'b mut NodeIndex {
        let n = name.to_owned();
        let insert = || deps.add_node(n);
        let node: &mut NodeIndex = names.entry(name).or_insert_with(insert);
        return node;
    }

    for fkey in fkeys {
        let src: NodeIndex = *ensure_node(fkey.table, &mut names, &mut deps);
        let target: NodeIndex = *ensure_node(fkey.foreign_table, &mut names, &mut deps);
        deps.add_edge(src, target, fkey.name.to_owned());
    }

    return deps;
}
