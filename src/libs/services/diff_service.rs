//! Service for computing text differences using 'similar' library.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use similar::{ChangeTag, TextDiff};

/// Represents a single change in the diff.
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub content: String,
    pub tag: ChangeTag,
}

#[derive(Clone)]
pub struct DiffService;

impl DiffService {
    /// Create a new DiffService.
    pub fn new() -> Self {
        Self
    }

    /// Compute difference between two text strings with optimizations.
    ///
    /// Uses efficient diff algorithm for better performance.
    ///
    /// # Arguments
    ///
    /// * `text_a` - Original text content
    /// * `text_b` - Modified text content
    ///
    /// # Returns
    ///
    /// * `Vec<DiffLine>` - Vector of diff changes with content and tags
    pub fn compute_diff(&self, text_a: &str, text_b: &str) -> Vec<DiffLine> {
        let diff = TextDiff::from_lines(text_a, text_b);

        diff.iter_all_changes()
            .map(|change| DiffLine {
                content: change.value().to_string(),
                tag: change.tag(),
            })
            .collect::<Vec<DiffLine>>()
    }
}
