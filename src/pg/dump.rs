use postgres;

pub fn dump_data_from_table(
    conn: &mut postgres::Client,
    schema: &str,
    table: Table,
) -> postgres::RowIter<'_> {
}
