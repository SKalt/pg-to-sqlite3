use std::usize;

use bit_vec::{self};
use chrono;
use postgres::{Column as PgColumn, Row as PgRow};
use postgres_types::{FromSql as FromPgSql, Type as PgType};
use rusqlite::{
    types::{Null as SqliteNull, Type as SqliteType},
    Error as SqliteError, ToSql as ToSqlite,
};
use serde_json;

pub fn pretty_relkind(relkind: &str) -> &str {
    match relkind {
        "r" => return "table",
        "v" => return "view",
        "m" => return "materialized view",
        "i" => return "index",
        "S" => return "sequence",
        "s" => return "special",
        "f" => return "foreign table",
        "p" => return "partitioned view",
        "I" => return "partitioned index",
        other => panic!("unexpected relkind {:?}", other),
    }
}
pub fn pretty_deptype(deptype: &str) -> &str {
    match deptype {
        "n" => return "normal",
        "a" => return "automatic",
        "i" => return "internal",
        "e" => return "extension",
        "p" => return "pinned",
        other => panic!("unknown deptype '{:?}'", other),
    }
}

pub fn get_pg_type_from_name(name: &str) -> Result<PgType, String> {
    match name {
        "bool" => Ok(PgType::BOOL),
        "bytea" => Ok(PgType::BYTEA),
        "char" => Ok(PgType::CHAR),
        "name" => Ok(PgType::NAME),
        "int8" => Ok(PgType::INT8),
        "int2" => Ok(PgType::INT2),
        "int2vector" => Ok(PgType::INT2_VECTOR),
        "int4" => Ok(PgType::INT4),
        "regproc" => Ok(PgType::REGPROC),
        "text" => Ok(PgType::TEXT),
        "oid" => Ok(PgType::OID),
        "tid" => Ok(PgType::TID),
        "xid" => Ok(PgType::XID),
        "cid" => Ok(PgType::CID),
        "oidvector" => Ok(PgType::OID_VECTOR),
        "pg_ddl_command" => Ok(PgType::PG_DDL_COMMAND),
        "json" => Ok(PgType::JSON),
        "xml" => Ok(PgType::XML),
        "_xml" => Ok(PgType::XML_ARRAY),
        "pg_node_tree" => Ok(PgType::PG_NODE_TREE),
        "_json" => Ok(PgType::JSON_ARRAY),
        "table_am_handler" => Ok(PgType::TABLE_AM_HANDLER),
        // "_xid8" => Ok(PgType::XID8_ARRAY),
        "index_am_handler" => Ok(PgType::INDEX_AM_HANDLER),
        "point" => Ok(PgType::POINT),
        "lseg" => Ok(PgType::LSEG),
        "path" => Ok(PgType::PATH),
        "box" => Ok(PgType::BOX),
        "polygon" => Ok(PgType::POLYGON),
        "line" => Ok(PgType::LINE),
        "_line" => Ok(PgType::LINE_ARRAY),
        "cidr" => Ok(PgType::CIDR),
        "_cidr" => Ok(PgType::CIDR_ARRAY),
        "float4" => Ok(PgType::FLOAT4),
        "float8" => Ok(PgType::FLOAT8),
        "unknown" => Ok(PgType::UNKNOWN),
        "circle" => Ok(PgType::CIRCLE),
        "_circle" => Ok(PgType::CIRCLE_ARRAY),
        "macaddr8" => Ok(PgType::MACADDR8),
        "_macaddr8" => Ok(PgType::MACADDR8_ARRAY),
        "money" => Ok(PgType::MONEY),
        "_money" => Ok(PgType::MONEY_ARRAY),
        "macaddr" => Ok(PgType::MACADDR),
        "inet" => Ok(PgType::INET),
        "_bool" => Ok(PgType::BOOL_ARRAY),
        "_bytea" => Ok(PgType::BYTEA_ARRAY),
        "_char" => Ok(PgType::CHAR_ARRAY),
        "_name" => Ok(PgType::NAME_ARRAY),
        "_int2" => Ok(PgType::INT2_ARRAY),
        "_int2vector" => Ok(PgType::INT2_VECTOR_ARRAY),
        "_int4" => Ok(PgType::INT4_ARRAY),
        "_regproc" => Ok(PgType::REGPROC_ARRAY),
        "_text" => Ok(PgType::TEXT_ARRAY),
        "_tid" => Ok(PgType::TID_ARRAY),
        "_xid" => Ok(PgType::XID_ARRAY),
        "_cid" => Ok(PgType::CID_ARRAY),
        "_oidvector" => Ok(PgType::OID_VECTOR_ARRAY),
        "_bpchar" => Ok(PgType::BPCHAR_ARRAY),
        "_varchar" => Ok(PgType::VARCHAR_ARRAY),
        "_int8" => Ok(PgType::INT8_ARRAY),
        "_point" => Ok(PgType::POINT_ARRAY),
        "_lseg" => Ok(PgType::LSEG_ARRAY),
        "_path" => Ok(PgType::PATH_ARRAY),
        "_box" => Ok(PgType::BOX_ARRAY),
        "_float4" => Ok(PgType::FLOAT4_ARRAY),
        "_float8" => Ok(PgType::FLOAT8_ARRAY),
        "_polygon" => Ok(PgType::POLYGON_ARRAY),
        "_oid" => Ok(PgType::OID_ARRAY),
        "aclitem" => Ok(PgType::ACLITEM),
        "_aclitem" => Ok(PgType::ACLITEM_ARRAY),
        "_macaddr" => Ok(PgType::MACADDR_ARRAY),
        "_inet" => Ok(PgType::INET_ARRAY),
        "bpchar" => Ok(PgType::BPCHAR),
        "varchar" => Ok(PgType::VARCHAR),
        "date" => Ok(PgType::DATE),
        "time" => Ok(PgType::TIME),
        "timestamp" => Ok(PgType::TIMESTAMP),
        "_timestamp" => Ok(PgType::TIMESTAMP_ARRAY),
        "_date" => Ok(PgType::DATE_ARRAY),
        "_time" => Ok(PgType::TIME_ARRAY),
        "timestamptz" => Ok(PgType::TIMESTAMPTZ),
        "_timestamptz" => Ok(PgType::TIMESTAMPTZ_ARRAY),
        "interval" => Ok(PgType::INTERVAL),
        "_interval" => Ok(PgType::INTERVAL_ARRAY),
        "_numeric" => Ok(PgType::NUMERIC_ARRAY),
        "_cstring" => Ok(PgType::CSTRING_ARRAY),
        "timetz" => Ok(PgType::TIMETZ),
        "_timetz" => Ok(PgType::TIMETZ_ARRAY),
        "bit" => Ok(PgType::BIT),
        "_bit" => Ok(PgType::BIT_ARRAY),
        "varbit" => Ok(PgType::VARBIT),
        "_varbit" => Ok(PgType::VARBIT_ARRAY),
        "numeric" => Ok(PgType::NUMERIC),
        "refcursor" => Ok(PgType::REFCURSOR),
        "_refcursor" => Ok(PgType::REFCURSOR_ARRAY),
        "regprocedure" => Ok(PgType::REGPROCEDURE),
        "regoper" => Ok(PgType::REGOPER),
        "regoperator" => Ok(PgType::REGOPERATOR),
        "regclass" => Ok(PgType::REGCLASS),
        "regtype" => Ok(PgType::REGTYPE),
        "_regprocedure" => Ok(PgType::REGPROCEDURE_ARRAY),
        "_regoper" => Ok(PgType::REGOPER_ARRAY),
        "_regoperator" => Ok(PgType::REGOPERATOR_ARRAY),
        "_regclass" => Ok(PgType::REGCLASS_ARRAY),
        "_regtype" => Ok(PgType::REGTYPE_ARRAY),
        "record" => Ok(PgType::RECORD),
        "cstring" => Ok(PgType::CSTRING),
        "any" => Ok(PgType::ANY),
        "anyarray" => Ok(PgType::ANYARRAY),
        "void" => Ok(PgType::VOID),
        "trigger" => Ok(PgType::TRIGGER),
        "language_handler" => Ok(PgType::LANGUAGE_HANDLER),
        "internal" => Ok(PgType::INTERNAL),
        "anyelement" => Ok(PgType::ANYELEMENT),
        "_record" => Ok(PgType::RECORD_ARRAY),
        "anynonarray" => Ok(PgType::ANYNONARRAY),
        "_txid_snapshot" => Ok(PgType::TXID_SNAPSHOT_ARRAY),
        "uuid" => Ok(PgType::UUID),
        "_uuid" => Ok(PgType::UUID_ARRAY),
        "txid_snapshot" => Ok(PgType::TXID_SNAPSHOT),
        "fdw_handler" => Ok(PgType::FDW_HANDLER),
        "pg_lsn" => Ok(PgType::PG_LSN),
        "_pg_lsn" => Ok(PgType::PG_LSN_ARRAY),
        "tsm_handler" => Ok(PgType::TSM_HANDLER),
        "pg_ndistinct" => Ok(PgType::PG_NDISTINCT),
        "pg_dependencies" => Ok(PgType::PG_DEPENDENCIES),
        "anyenum" => Ok(PgType::ANYENUM),
        "tsvector" => Ok(PgType::TS_VECTOR),
        "tsquery" => Ok(PgType::TSQUERY),
        "gtsvector" => Ok(PgType::GTS_VECTOR),
        "_tsvector" => Ok(PgType::TS_VECTOR_ARRAY),
        "_gtsvector" => Ok(PgType::GTS_VECTOR_ARRAY),
        "_tsquery" => Ok(PgType::TSQUERY_ARRAY),
        "regconfig" => Ok(PgType::REGCONFIG),
        "_regconfig" => Ok(PgType::REGCONFIG_ARRAY),
        "regdictionary" => Ok(PgType::REGDICTIONARY),
        "_regdictionary" => Ok(PgType::REGDICTIONARY_ARRAY),
        "jsonb" => Ok(PgType::JSONB),
        "_jsonb" => Ok(PgType::JSONB_ARRAY),
        "anyrange" => Ok(PgType::ANY_RANGE),
        "event_trigger" => Ok(PgType::EVENT_TRIGGER),
        "int4range" => Ok(PgType::INT4_RANGE),
        "_int4range" => Ok(PgType::INT4_RANGE_ARRAY),
        "numrange" => Ok(PgType::NUM_RANGE),
        "_numrange" => Ok(PgType::NUM_RANGE_ARRAY),
        "tsrange" => Ok(PgType::TS_RANGE),
        "_tsrange" => Ok(PgType::TS_RANGE_ARRAY),
        "tstzrange" => Ok(PgType::TSTZ_RANGE),
        "_tstzrange" => Ok(PgType::TSTZ_RANGE_ARRAY),
        "daterange" => Ok(PgType::DATE_RANGE),
        "_daterange" => Ok(PgType::DATE_RANGE_ARRAY),
        "int8range" => Ok(PgType::INT8_RANGE),
        "_int8range" => Ok(PgType::INT8_RANGE_ARRAY),
        "jsonpath" => Ok(PgType::JSONPATH),
        "_jsonpath" => Ok(PgType::JSONPATH_ARRAY),
        "regnamespace" => Ok(PgType::REGNAMESPACE),
        "_regnamespace" => Ok(PgType::REGNAMESPACE_ARRAY),
        "regrole" => Ok(PgType::REGROLE),
        "_regrole" => Ok(PgType::REGROLE_ARRAY),
        // "regcollation" => Ok(PgType::REGCOLLATION),
        // "_regcollation" => Ok(PgType::REGCOLLATION_ARRAY),
        "pg_mcv_list" => Ok(PgType::PG_MCV_LIST),
        // "pg_snapshot" => Ok(PgType::PG_SNAPSHOT),
        // "_pg_snapshot" => Ok(PgType::PG_SNAPSHOT_ARRAY),
        // "xid8" => Ok(PgType::XID8),
        // "anycompatible" => Ok(PgType::ANYCOMPATIBLE),
        // "anycompatiblearray" => Ok(PgType::ANYCOMPATIBLE_ARRAY),
        // "anycompatiblenonarray" => Ok(PgType::ANYCOMPATIBLENONARRAY),
        // "anycompatiblerange" => Ok(PgType::ANYCOMPATIBLE_RANGE),
        unknown => Err(format!("unknown or unprocessable column type {}", unknown)),
    }
}

pub fn sqlite_type_from_pg_type(pg_type: &PgType) -> Result<SqliteType, String> {
    match pg_type {
        &PgType::INT8 | &PgType::INT4 | &PgType::INT2 | &PgType::BOOL => Ok(SqliteType::Integer),

        &PgType::CHAR
        | &PgType::TEXT
        | &PgType::NAME
        | &PgType::VARCHAR
        | &PgType::BPCHAR
        | &PgType::UNKNOWN => Ok(SqliteType::Text),

        &PgType::JSON
        | &PgType::XML
        | &PgType::JSONB
        | &PgType::BIT
        | &PgType::VARBIT
        | &PgType::INT2_VECTOR
        | &PgType::BYTEA => Ok(SqliteType::Blob),

        &PgType::FLOAT4
        | &PgType::FLOAT8
        | &PgType::DATE
        | &PgType::TIME
        | &PgType::TIMESTAMP
        | &PgType::TIMESTAMPTZ
        | &PgType::TIMETZ
        | &PgType::TIMETZ_ARRAY
        | &PgType::NUMERIC => Ok(SqliteType::Real),
        unknown => Err(format!(
            "unable to convert postgres type {:?} to a sqlite type",
            unknown
        )),
    }
}

fn translate<'a, T>(row: &'a PgRow, index: usize) -> Box<dyn ToSqlite>
where
    T: ToSqlite,
    T: 'static, // TODO: explain why these bounds are needed, and what they are
    T: FromPgSql<'a>,
{
    return Box::new(row.get::<usize, T>(index));
}

fn translate_col<'a>(row: &'a PgRow, index: usize, col: &'a PgColumn) -> Box<dyn ToSqlite> {
    match col.type_() {
        &PgType::CHAR => translate::<'a, i8>(row, index),
        &PgType::INT2 => translate::<'a, i16>(row, index),
        &PgType::INT4 => translate::<'a, i32>(row, index),
        &PgType::INT8 => translate::<'a, i64>(row, index),
        &PgType::FLOAT4 | &PgType::FLOAT8 => translate::<'a, f64>(row, index),
        &PgType::BOOL => translate::<'a, bool>(row, index),
        &PgType::BYTEA => translate::<'a, Vec<u8>>(row, index),
        &PgType::TEXT | &PgType::NAME | &PgType::VARCHAR | &PgType::BPCHAR | &PgType::UNKNOWN => {
            translate::<'a, String>(row, index)
        }
        &PgType::JSON | &PgType::JSONB => translate::<'a, serde_json::Value>(row, index),
        &PgType::DATE => translate::<'a, chrono::NaiveDate>(row, index),
        &PgType::TIME => translate::<'a, chrono::NaiveTime>(row, index),
        // &PgType::TIMETZ ?
        &PgType::TIMESTAMP => translate::<'a, chrono::NaiveDateTime>(row, index),
        &PgType::TIMESTAMPTZ => translate::<'a, chrono::DateTime<chrono::Utc>>(row, index),
        &PgType::UUID => translate::<'a, uuid::Uuid>(row, index),
        &PgType::BIT | &PgType::VARBIT => {
            let bits: bit_vec::BitVec = row.get(index);
            let bytes: Vec<u8> = bits.to_bytes();
            return Box::new(bytes);
        }
        _ => unimplemented!(),
    }
}

pub fn translate_row(row: &PgRow) -> Vec<Box<dyn ToSqlite>> {
    return row
        .columns()
        .iter()
        .enumerate()
        .map(|(idx, col)| translate_col(row, idx, col))
        .collect();
}
