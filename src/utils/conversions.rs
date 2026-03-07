use anyhow::{Error, Result};
use chrono::{Duration, NaiveDate};
use std::path::Path;

/// Converts an Excel date-time float to a human-readable ISO 8601 string.
pub fn excel_datetime(excel_date: f64) -> Result<String, Error> {
    let base_date = NaiveDate::from_ymd_opt(1899, 12, 30) // Adjust for Excel leap year bug
        .ok_or_else(|| Error::msg("Invalid base date"))?;

    let days = excel_date.floor() as i64; // Whole days part
    let day_fraction = excel_date - days as f64; // Fractional day part
    let seconds = (86400.0 * day_fraction).round() as i64; // Convert fraction to seconds

    let date = base_date
        .checked_add_signed(Duration::days(days))
        .ok_or_else(|| Error::msg("Date calculation failed"))?;

    let datetime = date
        .and_hms_opt(0, 0, 0) // Start of the day
        .ok_or_else(|| Error::msg("Time calculation failed"))?
        .checked_add_signed(Duration::seconds(seconds)) // Add fractional day seconds
        .ok_or_else(|| Error::msg("DateTime calculation failed"))?;

    Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}

pub fn address_to_row_col(cell_address: &str) -> Result<(u32, u32), Error> {
    let split_at = cell_address
        .chars()
        .position(|c| c.is_ascii_digit())
        .ok_or_else(|| Error::msg("Invalid cell address format"))?;
    let (col_str, row_str) = cell_address.split_at(split_at);

    let col = col_str
        .chars()
        .rev()
        .enumerate()
        .try_fold(0u32, |acc, (i, c)| {
            if let Some(digit) = c.to_digit(36) {
                Ok(acc + (digit - 10) * 26u32.pow(i as u32))
            } else {
                Err(Error::msg("Invalid column label"))
            }
        })?;

    let row: u32 = row_str
        .parse()
        .map_err(|_| Error::msg("Invalid row number"))?;

    // Adjust for 0-based indexing used by Calamine
    Ok((row, col))
}

pub fn column_name_to_index(column_name: &str) -> Result<u32, Error> {
    let mut col_idx = 0;
    for (i, char) in column_name.chars().rev().enumerate() {
        let digit = char as u32 - 'A' as u32 + 1;
        col_idx += digit * 26u32.pow(i as u32);
    }
    Ok(col_idx - 1) // converting to 0-based index
}

// Helper function to extract the base filename without extension
pub fn extract_filename(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}
