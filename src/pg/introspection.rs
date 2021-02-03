use super::{
    ColInfo, FkeyConstraint, PkeyConstraint, Rel, Table, UniqueConstraint, View, ViewRelUsage,
};
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
            name: column_name.clone(),
            data_type: pg_type,
            nullable: (is_nullable == "YES"),
        };
        let table = tables.get_mut(&table_name).unwrap();
        table.column_order.push(column_name.clone());
        table.columns.insert(column_name, col);
    }
}

/// WARNING: postgres converts `CAST(thing AS TYPE)` to `thing::TYPE`, which sqlite can't handle.
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

// pub fn list_schemas(conn: &mut postgres::Client) -> Vec<String> {
//     // TODO: deprecate? We only need to check 1 schema.
//     return query::must_succeed(conn.query(
//         "
//         SELECT schema_name
//         FROM information_schema.schemata
//         WHERE schema_name NOT LIKE 'pg_%' AND schema_name != 'information_schema';
//         ",
//         &[],
//     ))
//     .iter()
//     .map(|row| row.get("schema_name"))
//     .collect();
// }

pub fn list_relations_in_schema(conn: &mut postgres::Client, schema_name: &str) -> Vec<Rel> {
    return query::must_succeed(conn.query(
        "
        SELECT
            c.oid
            , c.relname AS name
            , c.relkind::TEXT
            , c.reltuples::BIGINT AS approx_n_rows
            , pg_catalog.pg_get_userbyid(c.relowner) as owner
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
        let approx_n_rows = row.get("approx_n_rows");
        let relkind = pretty_relkind(row.get("relkind")).to_owned();
        return Rel {
            oid,
            name,
            relkind,
            approx_n_rows,
        };
    })
    .collect();
}

// TODO: parametrize with a Vec<str> schema names
pub(crate) fn get_all_fkey_constraints(
    conn: &mut postgres::Client,
    schema: &str,
) -> Vec<FkeyConstraint> {
    return query::must_succeed(conn.query(
        "
        SELECT tc.constraint_name,
            tc.constraint_type,
            tc.table_name,
            array_agg(kcu.column_name::TEXT) AS columns,
            ccu.table_name AS foreign_table_name,
            array_agg(ccu.column_name::TEXT) AS foreign_columns,
            tc.is_deferrable,
            tc.initially_deferred
        FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage AS ccu
                ON tc.constraint_name = ccu.constraint_name
        WHERE tc.constraint_schema = $1 AND tc.constraint_type = 'FOREIGN KEY'
        GROUP BY tc.constraint_name,
            tc.constraint_type,
            tc.table_name,
            tc.is_deferrable,
            tc.initially_deferred,
            ccu.table_name
        ",
        &[&schema],
    ))
    .iter()
    .map(|row| {
        let table = row.get("table_name");
        let col = row.get("columns");
        let constraint = row.get("constraint_name");
        let foreign_table = row.get("foreign_table_name");
        let foreign_columns = row.get("foreign_columns");
        return FkeyConstraint {
            table,
            columns: col,
            name: constraint,
            foreign_table,
            foreign_columns,
        };
    })
    .collect();
}

pub fn get_all_pkey_constraints(conn: &mut postgres::Client, schema: &str) -> Vec<PkeyConstraint> {
    return query::must_succeed(conn.query(
        "
        SELECT
            tc.constraint_name,
            tc.table_name,
            array_agg(kcu.column_name::TEXT) AS columns,
            tc.is_deferrable,
            tc.initially_deferred
        FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
        WHERE tc.constraint_schema = $1 AND tc.constraint_type = 'PRIMARY KEY'
        GROUP BY tc.constraint_name,
            tc.constraint_type,
            tc.table_name,
            tc.is_deferrable,
            tc.initially_deferred;
        ",
        &[&schema],
    ))
    .iter()
    .map(|row| {
        let name = row.get("constraint_name");
        let table = row.get("table_name");
        let columns = row.get("columns");
        return PkeyConstraint {
            name,
            table,
            columns,
        };
    })
    .collect();
}

pub fn get_all_unique_constraints(
    conn: &mut postgres::Client,
    schema: &str,
) -> Vec<UniqueConstraint> {
    return query::must_succeed(conn.query(
        "
        SELECT
            tc.constraint_name,
            tc.table_name,
            array_agg(kcu.column_name::TEXT) AS columns,
            tc.is_deferrable,
            tc.initially_deferred
        FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
        WHERE tc.constraint_schema = $1 AND tc.constraint_type = 'UNIQUE'
        GROUP BY tc.constraint_name,
            tc.constraint_type,
            tc.table_name,
            tc.is_deferrable,
            tc.initially_deferred;
        ",
        &[&schema],
    ))
    .iter()
    .map(|row| {
        let name = row.get("constraint_name");
        let table = row.get("table_name");
        let columns = row.get("columns");
        return UniqueConstraint {
            name,
            table,
            columns,
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

// fn list_view_dependencies(conn: &mut postgres::Client, schema: &str) -> Vec<ViewRelUsage> {
//     return query::must_succeed(conn.query(
//         "
//         SELECT DISTINCT
//             source_rel.oid AS source_oid,
//             source_rel.relname AS source_table,
//             dependent_rel.relname AS dependent_rel,
//             dependent_rel.oid AS dependent_oid
//         FROM pg_catalog.pg_depend AS dep
//         JOIN pg_catalog.pg_rewrite AS rewrite ON dep.objid = rewrite.oid
//         JOIN pg_catalog.pg_class AS dependent_rel ON rewrite.ev_class = dependent_rel.oid
//         JOIN pg_catalog.pg_class AS source_rel ON dep.refobjid = source_rel.oid
//         JOIN pg_catalog.pg_namespace source_ns ON source_ns.oid = source_rel.relnamespace
//         WHERE source_ns.nspname = $1
//             AND source_rel.oid <> dependent_rel.oid
//         ",
//         &[&schema],
//     ))
//     .iter()
//     .map(|row| {
//         let view_oid: u32 = row.get("src_oid");
//         let table_oid: u32 = row.get("dest_oid");
//         let view_name = row.get("view_name");
//         let table_name = row.get("table_name");
//         return ViewRelUsage {
//             view_oid,
//             view_name,
//             rel_name: table_name,
//             rel_oid: table_oid,
//         };
//     })
//     .collect();
// }
