use crate::utils::{conversions, manipulations};
use anyhow::{Error, Result};
use calamine::{Data, Range};
use indexmap::IndexMap;
use serde_json::{Map, Value};

pub fn extract_values(
    sheet: &Range<Data>,
    instructions: &Map<String, Value>,
) -> Result<IndexMap<String, Value>, Error> {
    let mut results = IndexMap::new();
    for (key, value) in instructions {
        match value {
            Value::Array(addresses) => {
                let mut address_values = Vec::new();
                for address_value in addresses {
                    let (row, col) = match address_value {
                        Value::String(cell_address) => {
                            conversions::address_to_row_col(cell_address)?
                        }
                        Value::Object(obj) => {
                            let row = obj
                                .get("row")
                                .and_then(Value::as_u64)
                                .ok_or_else(|| Error::msg("Missing 'row'"))?
                                as u32;
                            let col = obj
                                .get("col")
                                .and_then(Value::as_u64)
                                .ok_or_else(|| Error::msg("Missing 'col'"))?
                                as u32;
                            (row, col)
                        }
                        _ => return Err(Error::msg("Invalid or missing row/column specification")),
                    };
                    match manipulations::extract_cell_value(sheet, row, col, false) {
                        Ok((Some(cell_value), _)) => {
                            if !cell_value.is_null() {
                                address_values.push(cell_value);
                            }
                        }
                        Ok((None, _)) => (), // Ignore null values
                        Err(e) => return Err(e),
                    }
                }
                results.insert(key.clone(), Value::Array(address_values));
            }
            Value::String(cell_address) => {
                let (row, col) = conversions::address_to_row_col(cell_address)?;
                match manipulations::extract_cell_value(sheet, row, col, false) {
                    Ok((Some(cell_value), _)) => {
                        results.insert(key.clone(), cell_value);
                    }
                    Ok((None, _)) => {
                        results.insert(key.clone(), Value::Null);
                    }
                    Err(e) => return Err(e),
                }
            }
            _ => return Err(Error::msg("Invalid or missing row/column specification")),
        }
    }
    Ok(results)
}
