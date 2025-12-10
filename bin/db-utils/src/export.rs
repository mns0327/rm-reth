use crate::error::{DbUtilsError, Result};
use serde_json::{json, Value};
use storage::{StorageManager, TableId};
use types::Address;

pub fn export_tables(storage: &StorageManager, table_names: &[String]) -> Result<String> {
    let mut entries = json!({});

    for table_name in table_names {
        export_table(storage, table_name, &mut entries)?;
    }

    Ok(serde_json::to_string_pretty(&entries)?)
}

fn export_table(storage: &StorageManager, table_name: &str, entries: &mut Value) -> Result<()> {
    match table_name {
        "block" => {
            storage.get_ref(TableId::Block).to_block().with_read_transaction(|table| {
                let mut items = vec![];
                for result in table.range::<u64>(..)? {
                    let (k, v) = result?;
                    items.push(json!({
                        "id": k.value(),
                        "block": v.value(),
                    }));
                }
                entries["block"] = items.into();
                Ok(())
            })?;
        }
        "nonce" => {
            storage.get_ref(TableId::Nonce).to_nonce().with_read_transaction(|table| {
                let mut items = vec![];
                for result in table.range::<Address>(..)? {
                    let (k, v) = result?;
                    items.push(json!({
                        "address": k.value(),
                        "nonce": v.value(),
                    }));
                }
                entries["nonce"] = items.into();
                Ok(())
            })?;
        }
        "balance" => {
            storage.get_ref(TableId::Balance).to_balance().with_read_transaction(|table| {
                let mut items = vec![];
                for result in table.range::<Address>(..)? {
                    let (k, v) = result?;
                    items.push(json!({
                        "address": k.value(),
                        "balance": v.value(),
                    }));
                }
                entries["balance"] = items.into();
                Ok(())
            })?;
        }
        _ => return Err(DbUtilsError::InvalidTable(table_name.to_string())),
    }
    Ok(())
}