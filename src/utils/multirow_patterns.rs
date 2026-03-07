use crate::utils::parsed_config::{ParsedColumn, ParsedMultirowConfig};
use crate::utils::{conversions, helpers, manipulations};
use anyhow::{Error, Result};
use calamine::{Data, Range};
use indexmap::IndexMap;
use serde_json::{Map, Value};

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
    let stop_if_empty = match instructions.get("stop_if_empty") {
        Some(v) => v,
        None => return Ok(None),
    };

    let mut consecutive = instructions
        .get("stop_consecutive")
        .and_then(Value::as_u64)
        .unwrap_or(1) as usize;

    let (mode, columns) = match stop_if_empty {
        Value::String(s) => {
            if s == "row" {
                (StopMode::Row, vec![])
            } else {
                let col = conversions::column_name_to_index(s).map_err(|_| {
                    Error::msg(format!("Invalid column '{}' in 'stop_if_empty'", s))
                })?;
                (StopMode::Column, vec![col])
            }
        }
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
        Value::Object(obj) => {
            if let Some(mode_val) = obj.get("mode") {
                let mode_str = mode_val
                    .as_str()
                    .ok_or_else(|| Error::msg("'mode' in 'stop_if_empty' must be a string"))?;

                match mode_str {
                    "row" => {
                        if let Some(cons) = obj.get("consecutive").and_then(Value::as_u64) {
                            consecutive = cons as usize;
                        }
                        (StopMode::Row, vec![])
                    }
                    "column" => {
                        let column_val = obj.get("column").ok_or_else(|| {
                            Error::msg(
                                "'column' field required in 'stop_if_empty' when mode is 'column'",
                            )
                        })?;

                        if let Some(cons) = obj.get("consecutive").and_then(Value::as_u64) {
                            consecutive = cons as usize;
                        }

                        let cols = parse_column_value(column_val)?;
                        (StopMode::Column, cols)
                    }
                    _ => {
                        return Err(Error::msg(format!(
                            "Invalid mode '{}' in 'stop_if_empty'. Must be 'row' or 'column'",
                            mode_str
                        )))
                    }
                }
            } else {
                let column_val = obj.get("column").ok_or_else(|| {
                    Error::msg("'column' field required in 'stop_if_empty' object")
                })?;

                if let Some(cons) = obj.get("consecutive").and_then(Value::as_u64) {
                    consecutive = cons as usize;
                }

                let cols = parse_column_value(column_val)?;
                (StopMode::Column, cols)
            }
        }
        _ => {
            return Err(Error::msg(
                "'stop_if_empty' must be a string, array, or object",
            ))
        }
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
        Value::Array(arr) => arr
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let col_str = v.as_str().ok_or_else(|| {
                    Error::msg(format!(
                        "Invalid value in column array at position {}: expected string",
                        i
                    ))
                })?;
                conversions::column_name_to_index(col_str)
                    .map_err(|_| Error::msg(format!("Invalid column '{}'", col_str)))
            })
            .collect::<Result<Vec<_>, _>>(),
        _ => Err(Error::msg("'column' must be a string or array of strings")),
    }
}

/// Check if a row is empty using pre-computed column indices.
/// This version avoids re-parsing column names on every row check (Bottleneck #2 fix).
fn is_row_empty_with_parsed(
    sheet: &Range<Data>,
    row: u32,
    columns: &[ParsedColumn],
) -> Result<bool, Error> {
    for parsed_col in columns {
        for &col in &parsed_col.indices {
            match manipulations::extract_cell_value(sheet, row, col, false) {
                Ok((Some(value), _)) if !value.is_null() => {
                    return Ok(false);
                }
                _ => continue,
            }
        }
    }
    Ok(true)
}

fn are_columns_empty(sheet: &Range<Data>, row: u32, columns: &[u32]) -> Result<bool, Error> {
    for &col in columns {
        match manipulations::extract_cell_value(sheet, row, col, false) {
            Ok((Some(value), _)) if !value.is_null() => {
                if let Value::String(s) = &value {
                    if !s.trim().is_empty() {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            _ => continue,
        }
    }
    Ok(true)
}

/// Extract rows using pre-parsed configuration to avoid repeated parsing in the hot loop.
pub fn extract_rows(
    sheet: &Range<Data>,
    instructions: &Map<String, Value>,
) -> Result<Value, Error> {
    // Parse configuration ONCE before the row loop (Bottleneck #2, #3 fix)
    let config = ParsedMultirowConfig::from_instructions(instructions)?;
    let stop_config = parse_stop_config(instructions)?;

    let mut consecutive_empty = 0;

    if config.use_unique_id {
        // Return dictionary with unique_id as keys
        let mut results: IndexMap<String, Value> = IndexMap::new();

        for row in config.start_row..=config.end_row {
            // Check stop condition first
            if let Some(ref stop_cfg) = stop_config {
                let is_empty = match stop_cfg.mode {
                    StopMode::Row => is_row_empty_with_parsed(sheet, row, &config.columns)?,
                    StopMode::Column => are_columns_empty(sheet, row, &stop_cfg.columns)?,
                };

                if is_empty {
                    consecutive_empty += 1;
                    if consecutive_empty >= stop_cfg.consecutive {
                        break;
                    }
                    continue;
                } else {
                    consecutive_empty = 0;
                }
            }

            // Extract all parts of the composite unique_id using pre-computed indices
            let mut unique_id_parts = Vec::new();
            let mut all_parts_valid = true;

            for &col_index in &config.unique_id_indices {
                match manipulations::extract_cell_value(sheet, row, col_index, false) {
                    Ok((Some(value), _)) if value != Value::Null => {
                        // Conditional trim: only allocate if trimming changes the string (Bottleneck #5 fix)
                        let value_str = match &value {
                            Value::String(s) => {
                                let trimmed = s.trim();
                                if trimmed.len() == s.len() {
                                    s.clone()
                                } else {
                                    trimmed.to_string()
                                }
                            }
                            _ => value.to_string(),
                        };

                        if !value_str.is_empty() {
                            unique_id_parts.push(value_str);
                        } else {
                            all_parts_valid = false;
                            break;
                        }
                    }
                    _ => {
                        all_parts_valid = false;
                        break;
                    }
                }
            }

            if !all_parts_valid || unique_id_parts.is_empty() {
                if stop_config.is_none() {
                    consecutive_empty += 1;
                    if consecutive_empty >= 1 {
                        break;
                    }
                }
                continue;
            }

            consecutive_empty = 0;
            let unique_id_string = unique_id_parts.join(&config.unique_id_separator);

            // Extract column data using pre-computed indices (no parsing in this loop)
            let mut row_data = Map::new();
            for parsed_col in &config.columns {
                let mut cell_values = Vec::new();

                // Use pre-computed indices instead of parsing column names per row
                for &col in &parsed_col.indices {
                    match manipulations::extract_cell_value(sheet, row, col, false) {
                        Ok((Some(value), _)) if !value.is_null() => cell_values.push(value),
                        Ok((Some(_), _)) => (),
                        Ok((None, _)) => (),
                        Err(e) => return Err(e),
                    }
                }

                let final_value = match cell_values.len() {
                    0 => Value::Null,
                    1 => cell_values.pop().unwrap(),
                    _ => Value::Array(cell_values),
                };
                row_data.insert(parsed_col.name.clone(), final_value);
            }

            // Use helper function for unique key generation (Bottleneck #7 fix)
            let unique_key = helpers::make_unique_key_indexmap(&results, &unique_id_string);
            results.insert(unique_key, Value::Object(row_data));
        }

        Ok(Value::Object(results.into_iter().collect()))
    } else {
        // Return array of row objects
        let mut results = Vec::new();

        for row in config.start_row..=config.end_row {
            if let Some(ref stop_cfg) = stop_config {
                let is_empty = match stop_cfg.mode {
                    StopMode::Row => is_row_empty_with_parsed(sheet, row, &config.columns)?,
                    StopMode::Column => are_columns_empty(sheet, row, &stop_cfg.columns)?,
                };

                if is_empty {
                    consecutive_empty += 1;
                    if consecutive_empty >= stop_cfg.consecutive {
                        break;
                    }
                    continue;
                } else {
                    consecutive_empty = 0;
                }
            }

            let mut row_data = Map::new();
            let mut has_any_data = false;

            // Use pre-computed indices (no parsing in this loop)
            for parsed_col in &config.columns {
                let mut cell_values = Vec::new();

                for &col in &parsed_col.indices {
                    match manipulations::extract_cell_value(sheet, row, col, false) {
                        Ok((Some(value), _)) if !value.is_null() => {
                            cell_values.push(value);
                            has_any_data = true;
                        }
                        Ok((Some(_), _)) => (),
                        Ok((None, _)) => (),
                        Err(e) => return Err(e),
                    }
                }

                let final_value = match cell_values.len() {
                    0 => Value::Null,
                    1 => cell_values.pop().unwrap(),
                    _ => Value::Array(cell_values),
                };
                row_data.insert(parsed_col.name.clone(), final_value);
            }

            if stop_config.is_none() && !has_any_data {
                consecutive_empty += 1;
                if consecutive_empty >= 1 {
                    break;
                }
                continue;
            }

            if has_any_data {
                consecutive_empty = 0;
            }

            results.push(Value::Object(row_data));
        }

        Ok(Value::Array(results))
    }
}
