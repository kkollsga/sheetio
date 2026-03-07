use crate::utils::{conversions, manipulations};
use anyhow::{Error, Result};
use calamine::{Data, Range};
use indexmap::IndexMap;
use serde_json::{Map, Value};

pub fn extract_dataframe(
    sheet: &Range<Data>,
    instructions: &Map<String, Value>,
) -> Result<IndexMap<String, Value>, Error> {
    // Extracting row range
    let row_range = instructions
        .get("row_range")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::msg("Missing or invalid 'row_range'"))?;
    let start_row = row_range.first().and_then(Value::as_u64).unwrap_or(0) as u32;
    let end_row = row_range.get(1).and_then(Value::as_u64).unwrap_or(0) as u32;

    // Extracting column range
    let column_range = instructions
        .get("column_range")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::msg("Missing or invalid 'column_range'"))?;
    let start_column_index = match column_range
        .first()
        .ok_or_else(|| Error::msg("Invalid 'column_range'"))?
    {
        Value::Number(n) => n.as_u64().unwrap() as u32,
        Value::String(s) => conversions::column_name_to_index(s)?,
        _ => return Err(Error::msg("Invalid 'start_column' format")),
    };
    let end_column_index = match column_range
        .get(1)
        .ok_or_else(|| Error::msg("Invalid 'column_range'"))?
    {
        Value::Number(n) => n.as_u64().unwrap() as u32,
        Value::String(s) => conversions::column_name_to_index(s)?,
        _ => return Err(Error::msg("Invalid 'end_column' format")),
    };

    // Extracting header row
    let header_rows = instructions
        .get("header_row")
        .ok_or_else(|| Error::msg("Missing 'header_row'"))?;
    let header_indices: Vec<u32> = match header_rows {
        Value::Number(num) => vec![num.as_u64().unwrap() as u32],
        Value::Array(arr) => arr.iter().map(|v| v.as_u64().unwrap() as u32).collect(),
        _ => return Err(Error::msg("Invalid 'header_row' format")),
    };
    let separator = instructions
        .get("separator")
        .and_then(Value::as_str)
        .unwrap_or(" ");
    let mut dataframe: IndexMap<String, Value> = IndexMap::new();
    for i in start_column_index..=end_column_index {
        let header_string = manipulations::extract_headers(sheet, &header_indices, i, separator)?;

        let data_array = manipulations::extract_column_data(sheet, i, start_row, end_row)?;

        // Insert the header string and data array into the dataframe
        dataframe.insert(header_string, data_array);
    }

    Ok(dataframe)
}
