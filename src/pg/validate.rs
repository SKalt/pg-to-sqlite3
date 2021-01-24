use super::SchemaInformation;
use std::collections::HashSet;

pub fn validate_namespace(si: &SchemaInformation) -> Result<(), String> {
    let mut namespace = HashSet::new();
    // assert all tables are unique
    let mut duplicates: Vec<String> = vec![];
    for (name, _) in &si.tables {
        let present = namespace.insert(name);
        if present {
            duplicates.push(format!("duplicate table {}", name));
        }
    }
    for (name, _) in &si.views {
        let present = namespace.insert(name);
        if present {
            duplicates.push(format!("duplicate view {}", name));
        }
    }
    if duplicates.len() > 0 {
        return Err(format!(
            "{} relations have duplicate names:  \n{}",
            duplicates.len(),
            duplicates.join("\n  ")
        ));
    } else {
        return Ok(());
    }
}

pub fn validate_fkey_tables_present(si: &SchemaInformation) -> Result<(), String> {
    let mut missing = vec![];
    for (_, fk) in &si.fkey_constraints {
        if !si.tables.contains_key(&fk.table) {
            missing.push(format!("missing table {} from {}", fk.table, fk.name));
        }
        if !si.tables.contains_key(&fk.foreign_table) {
            missing.push(format!(
                "missing foreign table {} from {}",
                fk.foreign_table, fk.name
            ));
        }
    }
    if missing.len() == 0 {
        return Ok(());
    } else {
        return Err(format!(
            "Some tables referenced by constraints were missing:  \n{}",
            missing.join("\n  ")
        ));
    }
}
