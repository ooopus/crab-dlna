//! Network utilities for crab-dlna
//!
//! This module provides network-related utility functions,
//! including retry mechanisms and error handling.

use crate::config::MAX_NETWORK_RETRIES;
use log::{debug, warn};
use std::time::Duration;
use tokio::time::sleep;

/// Retries an async operation with exponential backoff
///
/// # Arguments
/// * `operation` - The async operation to retry
/// * `operation_name` - Name of the operation for logging
///
/// # Returns
/// Returns the result of the operation or the last error if all retries fail
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    operation_name: &str,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;

    for attempt in 1..=MAX_NETWORK_RETRIES {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("{operation_name} succeeded on attempt {attempt}");
                }
                return Ok(result);
            }
            Err(error) => {
                if attempt < MAX_NETWORK_RETRIES {
                    let delay = Duration::from_millis(100 * (1 << (attempt - 1))); // Exponential backoff
                    warn!(
                        "{operation_name} failed on attempt {attempt} ({error}), retrying in {delay:?}"
                    );
                    sleep(delay).await;
                } else {
                    warn!(
                        "{operation_name} failed on final attempt {attempt} ({error})"
                    );
                }
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap())
}
