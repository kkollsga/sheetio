use pyo3::prelude::*;
use pyo3::types::PyList;
use serde_json::to_string;
use std::sync::Arc;
use tokio::runtime;
mod parallel;
mod read_excel;
use parallel::process_files;
mod utils;
use utils::pylist_to_json;

#[pyfunction]
#[pyo3(signature = (file_paths, extraction_details, num_workers=None))]
fn excel_extract(
    _py: Python<'_>,
    file_paths: &Bound<'_, PyList>,
    extraction_details: &Bound<'_, PyList>,
    num_workers: Option<usize>,
) -> PyResult<String> {
    let file_paths: Vec<String> = file_paths
        .iter()
        .map(|p| {
            p.extract::<String>().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Error extracting string: {}", e))
            })
        })
        .collect::<PyResult<Vec<String>>>()?;

    let extraction_details_serde = pylist_to_json(extraction_details)?;
    // Wrap in Arc to avoid deep cloning for each file (Bottleneck #1 fix)
    let extraction_details_arc = Arc::new(extraction_details_serde);
    let rt = runtime::Runtime::new().unwrap();
    let results = rt
        .block_on(async {
            process_files(file_paths, extraction_details_arc, num_workers.unwrap_or(5)).await
        })
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Error processing files: {}", e))
        })?;

    // Serialize the entire Map<String, Value> into one JSON string
    let json_string = to_string(&results).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!(
            "Error converting results to JSON string: {}",
            e
        ))
    })?;

    Ok(json_string)
}

#[pymodule]
fn _sheet_excavator(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register the excel_extract function in the Python module
    m.add_function(wrap_pyfunction!(excel_extract, m)?)?;
    Ok(())
}
