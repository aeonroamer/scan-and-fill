use std::sync::Arc;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition, TableError};

pub type Db = Arc<Database>;

pub(super) type Table = TableDefinition<'static, &'static str, &'static str>;

pub(super) fn put(db: &Db, table: Table, key: &str, value: &str) -> Result<(), redb::Error> {
    let tx = db.begin_write()?;
    tx.open_table(table)?.insert(key, value)?;
    tx.commit()?;
    Ok(())
}

pub(super) fn remove(db: &Db, table: Table, key: &str) -> Result<(), redb::Error> {
    let tx = db.begin_write()?;
    tx.open_table(table)?.remove(key)?;
    tx.commit()?;
    Ok(())
}

pub(super) fn get(db: &Db, table: Table, key: &str) -> Result<Option<String>, redb::Error> {
    let tx = db.begin_read()?;
    let table = match tx.open_table(table) {
        Ok(table) => table,
        Err(TableError::TableDoesNotExist(_)) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    Ok(table.get(key)?.map(|value| value.value().to_string()))
}

pub(super) fn all(db: &Db, table: Table) -> Result<Vec<String>, redb::Error> {
    let tx = db.begin_read()?;
    let table = match tx.open_table(table) {
        Ok(table) => table,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    table
        .iter()?
        .map(|entry| {
            entry
                .map(|(_, value)| value.value().to_string())
                .map_err(redb::Error::from)
        })
        .collect()
}
