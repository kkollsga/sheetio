use anyhow::{Result, Error};
use calamine::{Range, Data};
use serde_json::{Map, Value};
use indexmap::IndexMap;
use crate::utils::{conversions, manipulations};

pub fn extract_rows(sheet: &Range<Data>, instructions: &Map<String, Value>) -> Result<IndexMap<String, Value>, Error> {
    let mut results = IndexMap::new();

    // Retrieve and parse the row_range array
    let row_range = instructions.get("row_range")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::msg("Missing or invalid 'row_range'"))?;
    let start_row = row_range.get(0).and_then(Value::as_u64)
        .ok_or_else(|| Error::msg("Missing 'start_row' in 'row_range'"))? as u32;  // Adjust for zero-based index
    let end_row = row_range.get(1).and_then(Value::as_u64)
        .ok_or_else(|| Error::msg("Missing 'end_row' in 'row_range'"))? as u32;  // Adjust for zero-based index

    let columns = instructions
        .get("columns")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::msg("Missing 'columns'"))?;

    // Parse unique_id - can be either a string or an array of strings
    let unique_id_value = instructions
        .get("unique_id")
        .ok_or_else(|| Error::msg("Missing 'unique_id'"))?;
    
    let unique_id_columns: Vec<String> = match unique_id_value {
        Value::String(s) => vec![s.clone()],
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err(Error::msg("'unique_id' array cannot be empty"));
            }
            arr.iter()
                .enumerate()
                .map(|(i, v)| {
                    v.as_str()
                        .ok_or_else(|| Error::msg(format!("Invalid value in 'unique_id' array at position {}: expected string", i)))
                        .map(|s| s.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?
        },
        _ => return Err(Error::msg("'unique_id' must be a string or an array of strings"))
    };

    // Convert column names to indices and validate
    let unique_id_indices: Vec<u32> = unique_id_columns
        .iter()
        .map(|col| conversions::column_name_to_index(col)
            .map_err(|_| Error::msg(format!("Invalid column '{}' in 'unique_id'", col))))
        .collect::<Result<Vec<_>, _>>()?;

    // Get the separator (default to "_")
    let separator = instructions
        .get("unique_id_separator")
        .and_then(Value::as_str)
        .unwrap_or("_");

    for row in start_row..=end_row {
        // Extract all parts of the composite unique_id
        let mut unique_id_parts = Vec::new();
        let mut all_parts_valid = true;
        
        for &col_index in &unique_id_indices {
            match manipulations::extract_cell_value(sheet, row, col_index, false) {
                Ok((Some(value), _)) if value != Value::Null => {
                    // Trim whitespace from string values
                    let value_str = match &value {
                        Value::String(s) => s.trim().to_string(),
                        _ => value.to_string(),
                    };
                    
                    if !value_str.is_empty() {
                        unique_id_parts.push(value_str);
                    } else {
                        // Empty string after trimming, treat as null
                        all_parts_valid = false;
                        break;
                    }
                },
                _ => {
                    // Null or missing value in any part - skip this row
                    all_parts_valid = false;
                    break;
                }
            }
        }
        
        // Skip row if any part of the unique_id is null/empty
        if !all_parts_valid || unique_id_parts.is_empty() {
            continue;
        }
        
        // Build the composite key
        let unique_id_string = unique_id_parts.join(separator);
        
        // Extract column data for this row
        let mut row_data = Map::new();
        for (column_name, column_index_value) in columns {
            let column_values = match column_index_value {
                Value::Array(arr) => arr.clone(),
                Value::String(s) => vec![Value::String(s.clone())],
                _ => return Err(Error::msg("Invalid column specification")),
            };

            let mut cell_values = Vec::new();
            for column_index_value in column_values {
                let column_index_str = match column_index_value {
                    Value::String(s) => s,
                    _ => return Err(Error::msg("Invalid column specification")),
                };

                let col = conversions::column_name_to_index(&column_index_str)?;
                match manipulations::extract_cell_value(sheet, row, col, false) {
                    Ok((Some(value), _)) if !value.is_null() => cell_values.push(value),
                    Ok((Some(_), _)) => (),  // Handle the case for non-null values that are not needed
                    Ok((None, _)) => (),     // Ignore when no value is found
                    Err(e) => return Err(e), // Propagate errors
                }
            }

            let final_value = match cell_values.len() {
                0 => Value::Null,
                1 => cell_values.pop().unwrap(),
                _ => Value::Array(cell_values),
            };
            row_data.insert(column_name.clone(), final_value);
        }

        // Handle duplicates with the existing _1, _2 pattern
        let mut unique_key = unique_id_string.clone();
        let mut counter = 1;
        while results.contains_key(&unique_key) {
            unique_key = format!("{}_{}", unique_id_string, counter);
            counter += 1;
        }
        results.insert(unique_key, Value::Object(row_data));
    }
    Ok(results)
}
