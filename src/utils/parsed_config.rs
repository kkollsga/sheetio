use serde_json::{Map, Value};
use anyhow::{Result, Error};
use crate::utils::conversions;

/// Pre-parsed column specification with indices already computed.
///
/// Instead of parsing column names like "A", "B" on every row iteration,
/// we pre-compute the numeric indices once during config parsing.
#[derive(Debug, Clone)]
pub struct ParsedColumn {
    /// The output key name for this column in the result JSON
    pub name: String,
    /// Pre-computed column indices (0-based). May have multiple indices
    /// when the column spec is an array like ["A", "B"] for merged values.
    pub indices: Vec<u32>,
}

/// Pre-parsed configuration for multirow extraction.
///
/// This struct holds all configuration values pre-parsed and validated,
/// eliminating repeated parsing and validation in the hot loop.
#[derive(Debug)]
pub struct ParsedMultirowConfig {
    /// Starting row number (1-based, as specified in config)
    pub start_row: u32,
    /// Ending row number (1-based, as specified in config)
    pub end_row: u32,
    /// Pre-parsed column specifications with computed indices
    pub columns: Vec<ParsedColumn>,
    /// Pre-computed column indices for unique_id (empty if no unique_id)
    pub unique_id_indices: Vec<u32>,
    /// Separator for joining composite unique_id parts
    pub unique_id_separator: String,
    /// Whether unique_id is being used
    pub use_unique_id: bool,
}

impl ParsedMultirowConfig {
    /// Parse and validate multirow extraction instructions upfront.
    ///
    /// This converts all column names to indices once, rather than
    /// on every row iteration.
    pub fn from_instructions(instructions: &Map<String, Value>) -> Result<Self, Error> {
        // Parse row_range
        let row_range = instructions.get("row_range")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::msg("Missing or invalid 'row_range'"))?;

        let start_row = row_range.get(0)
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::msg("Missing 'start_row' in 'row_range'"))? as u32;

        let end_row = row_range.get(1)
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::msg("Missing 'end_row' in 'row_range'"))? as u32;

        // Pre-parse columns - convert all column names to indices
        let columns_map = instructions.get("columns")
            .and_then(Value::as_object)
            .ok_or_else(|| Error::msg("Missing 'columns'"))?;

        let mut columns = Vec::with_capacity(columns_map.len());
        for (column_name, column_spec) in columns_map {
            let indices = Self::parse_column_spec(column_spec)?;
            columns.push(ParsedColumn {
                name: column_name.clone(),
                indices,
            });
        }

        // Pre-parse unique_id if present
        let use_unique_id = instructions.contains_key("unique_id");
        let unique_id_indices = if use_unique_id {
            Self::parse_unique_id(instructions.get("unique_id").unwrap())?
        } else {
            vec![]
        };

        let unique_id_separator = instructions
            .get("unique_id_separator")
            .and_then(Value::as_str)
            .unwrap_or("_")
            .to_string();

        Ok(Self {
            start_row,
            end_row,
            columns,
            unique_id_indices,
            unique_id_separator,
            use_unique_id,
        })
    }

    /// Parse a column specification which can be a string ("A") or array (["A", "B"]).
    fn parse_column_spec(spec: &Value) -> Result<Vec<u32>, Error> {
        match spec {
            Value::String(s) => {
                let idx = conversions::column_name_to_index(s)
                    .map_err(|_| Error::msg(format!("Invalid column '{}' in columns spec", s)))?;
                Ok(vec![idx])
            }
            Value::Array(arr) => {
                arr.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let col_str = v.as_str()
                            .ok_or_else(|| Error::msg(format!(
                                "Invalid value in column array at position {}: expected string", i
                            )))?;
                        conversions::column_name_to_index(col_str)
                            .map_err(|_| Error::msg(format!("Invalid column '{}'", col_str)))
                    })
                    .collect()
            }
            _ => Err(Error::msg("Column specification must be a string or array of strings"))
        }
    }

    /// Parse unique_id specification which can be a string or array of strings.
    fn parse_unique_id(value: &Value) -> Result<Vec<u32>, Error> {
        match value {
            Value::String(s) => {
                let idx = conversions::column_name_to_index(s)
                    .map_err(|_| Error::msg(format!("Invalid column '{}' in unique_id", s)))?;
                Ok(vec![idx])
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    return Err(Error::msg("'unique_id' array cannot be empty"));
                }
                arr.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let col_str = v.as_str()
                            .ok_or_else(|| Error::msg(format!(
                                "Invalid value in 'unique_id' array at position {}: expected string", i
                            )))?;
                        conversions::column_name_to_index(col_str)
                            .map_err(|_| Error::msg(format!("Invalid column '{}' in unique_id", col_str)))
                    })
                    .collect()
            }
            _ => Err(Error::msg("'unique_id' must be a string or an array of strings"))
        }
    }
}
