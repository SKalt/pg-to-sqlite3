use super::{ColInfo, FkeyConstraint, Rel, Table, View, ViewRelUsage};
use crate::pg::object_types::{get_pg_type_from_name, pretty_relkind};
use crate::pg::query;
use std::collections::HashMap;

pub fn get_table_defns(
    conn: &mut postgres::Client,
    tables: &mut HashMap<String, Table>,
    schema: &str,
) {
    let table_names: Vec<String> = tables.iter().map(|(name, _)| name.to_owned()).collect();
    let cols = query::must_succeed(conn.query(
        "
        SELECT
              col.column_name
            , col.ordinal_position
            , col.table_name
            , col.udt_name
            , col.is_nullable
            , col.character_maximum_length
            , col.character_octet_length
            , col.numeric_precision
            , col.numeric_precision_radix
            , col.numeric_scale
            , col.datetime_precision
            , col.interval_type
            , col.interval_precision
        FROM information_schema.columns col
        WHERE col.table_schema = $1 AND col.table_name = ANY($2)
        ORDER BY 2
        ",
        &[&schema, &table_names],
    ));
    for row in cols {
        let table_name: String = row.get("table_name");
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("udt_name");
        let is_nullable: &str = row.get("is_nullable");
        let pg_type = get_pg_type_from_name(&data_type).unwrap_or_else(|err| panic!(err));
        let col = ColInfo {
            name: column_name,
            data_type: pg_type,
            nullable: (is_nullable == "YES"),
        };
        let table = tables.get_mut(&table_name).unwrap();
        table.columns.push(col);
        // let data_type: String = row.get("data_type");
        // let ordinal_position: String = row.get("ordinal_position");
        // let character_maximum_length: String = row.get("character_maximum_length");
        // let character_octet_length: String = row.get("character_octet_length");
        // let numeric_precision: String = row.get("numeric_precision");
        // let numeric_precision_radix: String = row.get("numeric_precision_radix");
        // let numeric_scale: String = row.get("numeric_scale");
        // let datetime_precision: String = row.get("datetime_precision");
        // let interval_type: String = row.get("interval_type");
        // let interval_precision: String = row.get("interval_precision");
    }
}

pub fn get_view_defns(conn: &mut postgres::Client, views: &mut HashMap<String, View>) {
    let oids: Vec<u32> = views.iter().map(|(_, v)| v.oid).collect();
    let defns = query::must_succeed(conn.query(
        "
            SELECT (c.relname)::information_schema.sql_identifier AS name,
            pg_get_viewdef(c.oid)::information_schema.character_data AS defn
            FROM pg_catalog.pg_class AS c 
            WHERE c.oid = ANY($1)
        ",
        &[&oids],
    ));
    for row in defns {
        let name: String = row.get("name");
        let defn: String = row.get("defn");
        let view = views.get_mut(&name).unwrap();
        view.defn = defn;
    }
}

pub fn list_schemas(conn: &mut postgres::Client) -> Vec<String> {
    // TODO: deprecate? We only need to check 1 schema.
    return query::must_succeed(conn.query(
        "
        SELECT schema_name
        FROM information_schema.schemata
        WHERE schema_name NOT LIKE 'pg_%' AND schema_name != 'information_schema';
        ",
        &[],
    ))
    .iter()
    .map(|row| row.get("schema_name"))
    .collect();
}

pub fn list_relations_in_schema(conn: &mut postgres::Client, schema_name: &str) -> Vec<Rel> {
    return query::must_succeed(conn.query(
        "
        SELECT
            c.oid,
            c.relname AS name,
            c.relkind::TEXT,
            pg_catalog.pg_get_userbyid(c.relowner) as owner
        FROM pg_catalog.pg_class c
            LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind IN ('r','v','m','s','p')
            AND n.nspname = $1
            AND pg_catalog.pg_table_is_visible(c.oid)
        ORDER BY 1, 2;
        ",
        &[&schema_name],
    ))
    .iter()
    .map(|row| {
        let oid = row.get("oid");
        let name = row.get("name");
        let relkind = pretty_relkind(row.get("relkind")).to_owned();
        return Rel { oid, name, relkind };
    })
    .collect();
}

// TODO: parametrize with a Vec<str> schema names
pub fn list_all_fkey_constraints(conn: &mut postgres::Client, schema: &str) -> Vec<postgres::Row> {
    return query::must_succeed(conn.query(
        "
        SELECT
            table_contraints.constraint_name,
            table_contraints.table_name,
            key_column_usage.column_name,
            constraint_column_usage.table_name AS foreign_table_name,
            constraint_column_usage.column_name AS foreign_column_name
        FROM information_schema.table_constraints AS table_contraints
        JOIN
            information_schema.key_column_usage AS key_column_usage ON
            table_contraints.constraint_name = key_column_usage.constraint_name
        JOIN information_schema.constraint_column_usage AS constraint_column_usage ON
            constraint_column_usage.constraint_name = table_contraints.constraint_name
        WHERE constraint_type = 'FOREIGN KEY'
            AND table_contraints.constraint_schema = $1
        ",
        &[&schema],
    ));
}

pub(crate) fn get_all_fkey_constraints(
    conn: &mut postgres::Client,
    schema: &str,
) -> Vec<FkeyConstraint> {
    return list_all_fkey_constraints(conn, schema)
        .iter()
        .map(|row| {
            let table = row.get("table_name");
            // let table = get_name("table_name")(row);
            let col = row.get("column_name");
            let constraint = row.get("constraint_name");
            let foreign_table = row.get("foreign_table_name");
            let foreign_column = row.get("foreign_column_name");
            return FkeyConstraint {
                table,
                column: col,
                name: constraint,
                foreign_table,
                foreign_column,
            };
        })
        .collect();
}

pub(crate) fn get_view_refs(conn: &mut postgres::Client, schema: &str) -> Vec<ViewRelUsage> {
    return query::must_succeed(conn.query(
        "
        SELECT DISTINCT
            source_rel.oid        AS source_oid,
            source_rel.relname    AS source_table,
            dependent_rel.relname AS dependent_rel,
            dependent_rel.oid     AS dependent_oid
        FROM pg_catalog.pg_depend AS dep
        JOIN pg_catalog.pg_rewrite AS rewrite ON dep.objid = rewrite.oid
        JOIN pg_catalog.pg_class AS dependent_rel ON rewrite.ev_class = dependent_rel.oid
        JOIN pg_catalog.pg_class AS source_rel ON dep.refobjid = source_rel.oid
        JOIN pg_catalog.pg_namespace source_ns ON source_ns.oid = source_rel.relnamespace
        WHERE source_ns.nspname = $1 AND source_rel.oid <> dependent_rel.oid
        ",
        &[&schema], // TODO: parametrize schema
    ))
    .iter()
    .map(|row| {
        let view_oid: u32 = row.get("source_oid");
        let view_name = row.get("source_table");
        let rel_name = row.get("dependent_rel");
        let rel_oid: u32 = row.get("dependent_oid");
        return ViewRelUsage {
            view_oid,
            view_name,
            rel_name,
            rel_oid,
        };
    })
    .collect();
}

pub fn list_table_constraints(/* schema, table */) /* -> constraints */ {}

fn list_view_dependencies(conn: &mut postgres::Client, schema: &str) -> Vec<ViewRelUsage> {
    return query::must_succeed(conn.query(
        "
        SELECT DISTINCT
            source_rel.oid AS source_oid,
            source_rel.relname AS source_table,
            dependent_rel.relname AS dependent_rel,
            dependent_rel.oid AS dependent_oid
        FROM pg_catalog.pg_depend AS dep
        JOIN pg_catalog.pg_rewrite AS rewrite ON dep.objid = rewrite.oid
        JOIN pg_catalog.pg_class AS dependent_rel ON rewrite.ev_class = dependent_rel.oid
        JOIN pg_catalog.pg_class AS source_rel ON dep.refobjid = source_rel.oid
        JOIN pg_catalog.pg_namespace source_ns ON source_ns.oid = source_rel.relnamespace
        WHERE source_ns.nspname = $1
            AND source_rel.oid <> dependent_rel.oid
        ",
        &[&schema],
    ))
    .iter()
    .map(|row| {
        let view_oid: u32 = row.get("src_oid");
        let table_oid: u32 = row.get("dest_oid");
        let view_name = row.get("view_name");
        let table_name = row.get("table_name");
        return ViewRelUsage {
            view_oid,
            view_name,
            rel_name: table_name,
            rel_oid: table_oid,
        };
    })
    .collect();
}

/// may fail if more than 4.5 billion rows
pub fn count_rows_in_table(conn: &mut postgres::Client, schema: &str, table: &str) -> u32 {
    let n = query::must_succeed(conn.query("SELECT count(*) AS n FROM {}.{}", &[&schema, &table]))
        .first()
        .unwrap()
        .get("n");
    return n;
}
