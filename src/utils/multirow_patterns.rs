use anyhow::{Result, Error};
use calamine::{Range, Data};
use serde_json::{Map, Value};
use indexmap::IndexMap;
use crate::utils::{conversions, manipulations};

#[derive(Debug)]
enum StopMode {
    Column,
    Row,
}

#[derive(Debug)]
struct StopConfig {
    mode: StopMode,
    columns: Vec<u32>,
    consecutive: usize,
}

fn parse_stop_config(instructions: &Map<String, Value>) -> Result<Option<StopConfig>, Error> {
    // Check for stop_if_empty parameter
    let stop_if_empty = match instructions.get("stop_if_empty") {
        Some(v) => v,
        None => return Ok(None),
    };

    // Parse consecutive threshold (can come from stop_consecutive or from object)
    let mut consecutive = instructions
        .get("stop_consecutive")
        .and_then(Value::as_u64)
        .unwrap_or(1) as usize;

    let (mode, columns) = match stop_if_empty {
        // Simple string form: "A"
        Value::String(s) => {
            if s == "row" {
                (StopMode::Row, vec![])
            } else {
                let col = conversions::column_name_to_index(s)
                    .map_err(|_| Error::msg(format!("Invalid column '{}' in 'stop_if_empty'", s)))?;
                (StopMode::Column, vec![col])
            }
        }
        // Array form: ["A", "B"]
        Value::Array(arr) => {
            let cols = arr
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let col_str = v.as_str()
                        .ok_or_else(|| Error::msg(format!("Invalid value in 'stop_if_empty' array at position {}: expected string", i)))?;
                    conversions::column_name_to_index(col_str)
                        .map_err(|_| Error::msg(format!("Invalid column '{}' in 'stop_if_empty'", col_str)))
                })
                .collect::<Result<Vec<_>, _>>()?;
            (StopMode::Column, cols)
        }
        // Object form: {column: "A", consecutive: 2} or {mode: "row", consecutive: 2}
        Value::Object(obj) => {
            // Check if mode is specified
            if let Some(mode_val) = obj.get("mode") {
                let mode_str = mode_val.as_str()
                    .ok_or_else(|| Error::msg("'mode' in 'stop_if_empty' must be a string"))?;

                match mode_str {
                    "row" => {
                        // Override consecutive if specified in object
                        if let Some(cons) = obj.get("consecutive").and_then(Value::as_u64) {
                            consecutive = cons as usize;
                        }
                        (StopMode::Row, vec![])
                    }
                    "column" => {
                        // Column mode requires 'column' field
                        let column_val = obj.get("column")
                            .ok_or_else(|| Error::msg("'column' field required in 'stop_if_empty' when mode is 'column'"))?;

                        if let Some(cons) = obj.get("consecutive").and_then(Value::as_u64) {
                            consecutive = cons as usize;
                        }

                        let cols = parse_column_value(column_val)?;
                        (StopMode::Column, cols)
                    }
                    _ => return Err(Error::msg(format!("Invalid mode '{}' in 'stop_if_empty'. Must be 'row' or 'column'", mode_str)))
                }
            } else {
                // Backward compatibility: if no mode, assume column mode
                let column_val = obj.get("column")
                    .ok_or_else(|| Error::msg("'column' field required in 'stop_if_empty' object"))?;

                if let Some(cons) = obj.get("consecutive").and_then(Value::as_u64) {
                    consecutive = cons as usize;
                }

                let cols = parse_column_value(column_val)?;
                (StopMode::Column, cols)
            }
        }
        _ => return Err(Error::msg("'stop_if_empty' must be a string, array, or object"))
    };

    Ok(Some(StopConfig {
        mode,
        columns,
        consecutive,
    }))
}

fn parse_column_value(column_val: &Value) -> Result<Vec<u32>, Error> {
    match column_val {
        Value::String(s) => {
            let col = conversions::column_name_to_index(s)
                .map_err(|_| Error::msg(format!("Invalid column '{}' in 'stop_if_empty'", s)))?;
            Ok(vec![col])
        }
        Value::Array(arr) => {
            arr.iter()
                .enumerate()
                .map(|(i, v)| {
                    let col_str = v.as_str()
                        .ok_or_else(|| Error::msg(format!("Invalid value in column array at position {}: expected string", i)))?;
                    conversions::column_name_to_index(col_str)
                        .map_err(|_| Error::msg(format!("Invalid column '{}'", col_str)))
                })
                .collect::<Result<Vec<_>, _>>()
        }
        _ => Err(Error::msg("'column' must be a string or array of strings"))
    }
}

fn is_row_empty(sheet: &Range<Data>, row: u32, columns: &Map<String, Value>) -> Result<bool, Error> {
    // Check if ALL data columns are empty
    for (_, column_index_value) in columns {
        let column_values = match column_index_value {
            Value::Array(arr) => arr.clone(),
            Value::String(s) => vec![Value::String(s.clone())],
            _ => return Err(Error::msg("Invalid column specification")),
        };

        for column_index_value in column_values {
            let column_index_str = match column_index_value {
                Value::String(s) => s,
                _ => return Err(Error::msg("Invalid column specification")),
            };

            let col = conversions::column_name_to_index(&column_index_str)?;
            match manipulations::extract_cell_value(sheet, row, col, false) {
                Ok((Some(value), _)) if !value.is_null() => {
                    // Found non-null value, row is not empty
                    return Ok(false);
                }
                _ => continue,
            }
        }
    }
    // All columns are null/empty
    Ok(true)
}

fn are_columns_empty(sheet: &Range<Data>, row: u32, columns: &[u32]) -> Result<bool, Error> {
    // Check if ALL specified columns are empty
    for &col in columns {
        match manipulations::extract_cell_value(sheet, row, col, false) {
            Ok((Some(value), _)) if !value.is_null() => {
                // Check for empty string after trimming
                if let Value::String(s) = &value {
                    if !s.trim().is_empty() {
                        return Ok(false); // Found non-empty value
                    }
                } else {
                    return Ok(false); // Found non-null value
                }
            }
            _ => continue,
        }
    }
    // All specified columns are null/empty
    Ok(true)
}

pub fn extract_rows(sheet: &Range<Data>, instructions: &Map<String, Value>) -> Result<Value, Error> {
    // Retrieve and parse the row_range array
    let row_range = instructions.get("row_range")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::msg("Missing or invalid 'row_range'"))?;
    let start_row = row_range.get(0).and_then(Value::as_u64)
        .ok_or_else(|| Error::msg("Missing 'start_row' in 'row_range'"))? as u32;
    let end_row = row_range.get(1).and_then(Value::as_u64)
        .ok_or_else(|| Error::msg("Missing 'end_row' in 'row_range'"))? as u32;

    let columns = instructions
        .get("columns")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::msg("Missing 'columns'"))?;

    // Parse stop_if_empty configuration
    let stop_config = parse_stop_config(instructions)?;

    // Parse unique_id - now optional
    let use_unique_id = instructions.contains_key("unique_id");
    let (_unique_id_columns, unique_id_indices, separator) = if use_unique_id {
        let unique_id_value = instructions.get("unique_id").unwrap();

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

        (unique_id_columns, unique_id_indices, separator)
    } else {
        (vec![], vec![], "")
    };

    let mut consecutive_empty = 0;

    if use_unique_id {
        // Return dictionary with unique_id as keys
        let mut results = IndexMap::new();

        for row in start_row..=end_row {
            // Check stop condition first
            if let Some(ref config) = stop_config {
                let is_empty = match config.mode {
                    StopMode::Row => is_row_empty(sheet, row, columns)?,
                    StopMode::Column => are_columns_empty(sheet, row, &config.columns)?,
                };

                if is_empty {
                    consecutive_empty += 1;
                    if consecutive_empty >= config.consecutive {
                        break; // Abort processing
                    }
                    continue; // Skip this empty row
                } else {
                    consecutive_empty = 0; // Reset counter
                }
            }

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
                // If no explicit stop_config, count this as empty for implicit stopping
                if stop_config.is_none() {
                    consecutive_empty += 1;
                    if consecutive_empty >= 1 {
                        break; // Default: stop after 1 row with empty unique_id
                    }
                }
                continue;
            }

            // Reset counter when we find valid data
            consecutive_empty = 0;

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

        Ok(Value::Object(results.into_iter().collect()))
    } else {
        // Return array of row objects
        let mut results = Vec::new();

        for row in start_row..=end_row {
            // Check stop condition
            if let Some(ref config) = stop_config {
                let is_empty = match config.mode {
                    StopMode::Row => is_row_empty(sheet, row, columns)?,
                    StopMode::Column => are_columns_empty(sheet, row, &config.columns)?,
                };

                if is_empty {
                    consecutive_empty += 1;
                    if consecutive_empty >= config.consecutive {
                        break; // Abort processing
                    }
                    continue; // Skip this empty row
                } else {
                    consecutive_empty = 0; // Reset counter
                }
            }

            // Extract column data for this row
            let mut row_data = Map::new();
            let mut has_any_data = false;

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
                        Ok((Some(value), _)) if !value.is_null() => {
                            cell_values.push(value);
                            has_any_data = true;
                        }
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

            // If no stop_config and row is completely empty, stop by default
            if stop_config.is_none() && !has_any_data {
                consecutive_empty += 1;
                if consecutive_empty >= 1 {
                    break; // Default: stop after 1 completely empty row
                }
                continue;
            }

            // Reset counter when we find data
            if has_any_data {
                consecutive_empty = 0;
            }

            results.push(Value::Object(row_data));
        }

        Ok(Value::Array(results))
    }
}
