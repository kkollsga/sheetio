use serde_json::{Map, Value};
use indexmap::IndexMap;

/// Generates a unique key by appending _1, _2, etc. if the base key already exists.
///
/// Used to handle duplicate keys in extraction results without overwriting data.
///
/// # Arguments
/// * `existing` - The map to check for existing keys
/// * `base` - The base key name to make unique
///
/// # Returns
/// A unique key that doesn't exist in the map
pub fn make_unique_key(existing: &Map<String, Value>, base: &str) -> String {
    if !existing.contains_key(base) {
        return base.to_string();
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}_{}", base, counter);
        if !existing.contains_key(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// IndexMap variant for multirow_patterns which uses IndexMap to preserve insertion order.
///
/// # Arguments
/// * `existing` - The IndexMap to check for existing keys
/// * `base` - The base key name to make unique
///
/// # Returns
/// A unique key that doesn't exist in the IndexMap
pub fn make_unique_key_indexmap(existing: &IndexMap<String, Value>, base: &str) -> String {
    if !existing.contains_key(base) {
        return base.to_string();
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}_{}", base, counter);
        if !existing.contains_key(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}
