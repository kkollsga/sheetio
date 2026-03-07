use crate::read_excel::process_file;
use crate::utils::{conversions, helpers};
use anyhow::{Error, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use serde_json::{Map, Value};
use std::{sync::Arc, time::Instant};
use tokio::sync::Semaphore;

/// Process multiple files in parallel with a semaphore-controlled worker pool.
///
/// Uses Arc<Vec<Value>> to share extraction config without deep cloning (Bottleneck #1 fix).
pub async fn process_files(
    file_paths: Vec<String>,
    extraction_details: Arc<Vec<Value>>,
    num_workers: usize,
) -> Result<Map<String, Value>, Error> {
    println!("Processing files!");
    let semaphore = Arc::new(Semaphore::new(num_workers));

    let mut futures = FuturesUnordered::new();
    let start_time = Instant::now();
    let total = file_paths.len();

    for (index, path_str) in file_paths.into_iter().enumerate() {
        let path_str_clone = path_str.clone();
        // Use Arc::clone for cheap reference count increment instead of deep clone
        let details_clone = Arc::clone(&extraction_details);
        let sem_clone = semaphore.clone();

        let permit = sem_clone.acquire_owned().await.unwrap();

        futures.push(tokio::spawn(async move {
            let result = process_file(path_str_clone, details_clone).await;
            let files_left = total - (index + 1);
            let avg_time_per_file = if index > 0 {
                start_time.elapsed().as_secs_f64() / (index + 1) as f64
            } else {
                0.0
            };
            let estimated_time_left = avg_time_per_file * files_left as f64;
            println!(
                "Progress: {}/{} files. Avg: {:.2}s. Time left: {:.2}s.",
                (index + 1),
                total,
                avg_time_per_file,
                estimated_time_left
            );
            drop(permit);
            result
        }));
    }

    let mut results = Map::new();
    while let Some(res) = futures.next().await {
        match res {
            Ok(Ok(value)) => {
                if let Some(file_path) = value.get("filepath").and_then(|v| v.as_str()) {
                    let base_filename = conversions::extract_filename(file_path);
                    // Use helper function for unique key generation (Bottleneck #7 fix)
                    let filename_key = helpers::make_unique_key(&results, &base_filename);
                    results.insert(filename_key, value);
                }
            }
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(anyhow::Error::new(e)),
        }
    }

    println!(
        "All files processed. Total time: {:.2?}",
        start_time.elapsed()
    );
    Ok(results)
}
