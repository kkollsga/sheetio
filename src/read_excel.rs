use calamine::{Reader, open_workbook_auto};
use serde_json::{Map, Value};
use anyhow::{Result, Error};
use std::iter::Iterator;
use crate::utils::{conversions, manipulations, dataframe, single_cells, multirow_patterns, match_sheet_names};

fn extend_unique<T: PartialEq>(vec: &mut Vec<T>, value: T) {
    if !vec.contains(&value) {
        vec.push(value);
    }
}

pub async fn process_file(file_path: String, extraction_details: Vec<Value>) -> Result<Value, Error> {
    let mut results = Map::new();
    results.insert("filepath".to_string(), Value::String(file_path.clone()));

    for extract in extraction_details.iter() {
        let map = match extract {
            Value::Object(map) => map,
            _ => return Err(Error::msg("Extraction detail should be a JSON object")),
        };

        let mut workbook = match open_workbook_auto(&file_path) {
            Ok(workbook) => workbook,
            Err(err) => {
                let base_filename = conversions::extract_filename(&file_path);
                println!("Error: {} :: {}", base_filename, err);
                return Ok(Value::Null); // or return an empty object, depending on your needs
            }
        };

        let mut sheet_names: Vec<String> = Vec::new();
        if let Some(sheets) = map.get("sheets") {
            if let Some(sheets_array) = sheets.as_array() {
                let skip_sheets = map.get("skip_sheets")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().cloned().collect::<Vec<_>>())
                    .unwrap_or_else(|| Vec::new());

                for sheet in sheets_array {
                    if let Some(sheet_str) = sheet.as_str() {
                        if sheet_str.contains('*') {
                            for sheet_name in match_sheet_names(&workbook.sheet_names().to_vec(), sheet_str) {
                                if !skip_sheets.iter().any(|s| s == &sheet_name) {
                                    extend_unique(&mut sheet_names, sheet_name);
                                }
                            }
                        } else {
                            let sheet_name = sheet_str.to_string();
                            if !skip_sheets.iter().any(|s| s == &sheet_name) {
                                extend_unique(&mut sheet_names, sheet_name);
                            }
                        }
                    } else {
                        return Err(Error::msg("Invalid sheet name"));
                    }
                }
            } else {
                return Err(Error::msg("Invalid \"sheets\" value in extraction details"));
            }
        } else {
            return Err(Error::msg("Missing \"sheets\" key in extraction details"));
        }

        let break_if_null = map.get("break_if_null").and_then(|f| f.as_str());
        let extractions = map
            .get("extractions")
            .and_then(|extr| extr.as_array())
            .ok_or_else(|| Error::msg("Missing or invalid \"extractions\" key in extraction details"))?
            .iter()
            .map(|extr| {
                let obj = extr.as_object().ok_or_else(|| Error::msg("Each extraction should be a JSON object"))?;
                let function = obj.get("function")
                    .and_then(|f| f.as_str())
                    .ok_or_else(|| Error::msg("Missing 'function' key"))?
                    .to_string();
                let function_label = obj.get("label")
                    .and_then(|f| f.as_str())
                    .unwrap_or("") // Provide "" as default if 'label' is missing
                    .to_string();
                let instructions = obj.get("instructions")
                    .and_then(|i| i.as_object())
                    .cloned()
                    .ok_or_else(|| Error::msg("Missing 'instructions' key"))?;
                Ok((function, function_label, instructions))
            })
            .collect::<Result<Vec<(String, String, Map<String, Value>)>, Error>>()?;
        for sheet_name in &sheet_names {
            let sheet = match workbook.worksheet_range(sheet_name) {
                Ok(sheet) => sheet,
                Err(_) => {
                    let base_filename = conversions::extract_filename(&file_path);
                    println!("{}: Sheet '{}' not found, skipping extraction.", base_filename, sheet_name);
                    continue;
                }
            };

            if let Some(break_if_null_value) = break_if_null {
                let (row, col) = conversions::address_to_row_col(break_if_null_value)?;
                let (cell_value, _) = manipulations::extract_cell_value(&sheet, row, col, false)?;
                if let Some(value) = cell_value {
                    if value.is_null() {
                        break; // Break out of the sheet loop
                    }
                } else {
                    break;
                }
            }

            let mut sheet_results = Map::new();
            for (function, label, instructions) in &extractions {
                match function.as_str() {
                    "single_cells" => {
                        let cells_object = single_cells::extract_values(&sheet, &instructions)?;

                        if label.is_empty() {
                            for (key, value) in cells_object {
                                let mut unique_key = key.clone();
                                let mut counter = 1;
                                while sheet_results.contains_key(&unique_key) {
                                    unique_key = format!("{}_{}", key, counter);
                                    counter += 1;
                                }
                                sheet_results.insert(unique_key, value);
                            }
                        } else {
                            if let Some(Value::Object(existing_map)) = sheet_results.get_mut(label.as_str()) {
                                for (key, value) in cells_object {
                                    existing_map.insert(key, value);
                                }
                            } else {
                                let value: Value = cells_object.into_iter().collect();
                                sheet_results.insert(label.clone(), value);
                            }
                        }
                    }
                    "multirow_patterns" => {
                        let result_value = multirow_patterns::extract_rows(&sheet, &instructions)?;

                        if label.is_empty() {
                            // If no label, try to merge the result into sheet_results
                            match result_value {
                                Value::Object(obj) => {
                                    for (key, value) in obj {
                                        let mut unique_key = key.clone();
                                        let mut counter = 1;
                                        while sheet_results.contains_key(&unique_key) {
                                            unique_key = format!("{}_{}", key, counter);
                                            counter += 1;
                                        }
                                        sheet_results.insert(unique_key, value);
                                    }
                                }
                                Value::Array(_) => {
                                    // For array results without label, use function name as key
                                    sheet_results.insert("multirow_patterns".to_string(), result_value);
                                }
                                _ => {}
                            }
                        } else {
                            // With label, store the result directly under the label
                            if let Some(Value::Object(existing_map)) = sheet_results.get_mut(label.as_str()) {
                                // If label already exists as object, try to merge
                                match result_value {
                                    Value::Object(obj) => {
                                        for (key, value) in obj {
                                            existing_map.insert(key, value);
                                        }
                                    }
                                    _ => {
                                        // Can't merge non-object into existing object, replace with new value
                                        sheet_results.insert(label.clone(), result_value);
                                    }
                                }
                            } else {
                                sheet_results.insert(label.clone(), result_value);
                            }
                        }
                    }
                    "dataframe" => {
                        let cells_object = dataframe::extract_dataframe(&sheet, &instructions)?;

                        if label.is_empty() {
                            for (key, value) in cells_object {
                                let mut unique_key = key.clone();
                                let mut counter = 1;
                                while sheet_results.contains_key(&unique_key) {
                                    unique_key = format!("{}_{}", key, counter);
                                    counter += 1;
                                }
                                sheet_results.insert(unique_key, value);
                            }
                        } else {
                            if let Some(Value::Object(existing_map)) = sheet_results.get_mut(label.as_str()) {
                                for (key, value) in cells_object {
                                    existing_map.insert(key, value);
                                }
                            } else {
                                let value: Value = cells_object.into_iter().collect();
                                sheet_results.insert(label.clone(), value);
                            }
                        }
                    }
                    _ => {
                        println!("Unsupported function type '{}'", function);
                        continue;
                    }
                }
            }

            if let Some(Value::Object(existing_map)) = results.get_mut(&sheet_name.to_string()) {
                for (key, value) in sheet_results {
                    existing_map.insert(key, value);
                }
            } else {
                results.insert(sheet_name.to_string(), Value::Object(sheet_results));
            }
        }
    }

    Ok(Value::Object(results))
}
