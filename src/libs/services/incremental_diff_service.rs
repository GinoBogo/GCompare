//! Incremental diff service for real-time line-by-line difference detection.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use similar::{ChangeTag, TextDiff};

/// Result of an incremental diff operation
#[derive(Debug, Clone)]
pub struct IncrementalDiffResult {
    pub changed_lines_a: Vec<usize>,
    pub changed_lines_b: Vec<usize>,
    pub empty_lines_a: Vec<usize>,
    pub empty_lines_b: Vec<usize>,
}

#[derive(Clone)]
pub struct IncrementalDiffService;

impl IncrementalDiffService {
    /// Create a new IncrementalDiffService.
    pub fn new() -> Self {
        Self
    }

    /// Compute line-by-line differences with efficient change tracking.
    /// Returns line numbers that have differences.
    pub fn compute_line_diff(&self, text_a: &str, text_b: &str) -> IncrementalDiffResult {
        let _lines_a: Vec<&str> = text_a.lines().collect();
        let _lines_b: Vec<&str> = text_b.lines().collect();

        let diff = TextDiff::from_lines(text_a, text_b);

        let mut changed_lines_a = Vec::new();
        let mut changed_lines_b = Vec::new();
        let mut empty_lines_a = Vec::new();
        let mut empty_lines_b = Vec::new();

        let mut line_a_index = 0;
        let mut line_b_index = 0;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    line_a_index += 1;
                    line_b_index += 1;
                }
                ChangeTag::Delete => {
                    // Check if the deleted content is empty or whitespace-only
                    let is_empty_or_whitespace = change.value().trim().is_empty();

                    if is_empty_or_whitespace {
                        empty_lines_a.push(line_a_index);
                    } else {
                        changed_lines_a.push(line_a_index);
                    }
                    line_a_index += 1;
                }
                ChangeTag::Insert => {
                    // Check if the inserted content is empty or whitespace-only
                    let is_empty_or_whitespace = change.value().trim().is_empty();

                    if is_empty_or_whitespace {
                        empty_lines_b.push(line_b_index);
                    } else {
                        changed_lines_b.push(line_b_index);
                    }
                    line_b_index += 1;
                }
            }
        }

        IncrementalDiffResult {
            changed_lines_a,
            changed_lines_b,
            empty_lines_a,
            empty_lines_b,
        }
    }

    /// Compute diff for a specific line range only (more efficient for real-time).
    #[allow(dead_code)]
    pub fn compute_line_range_diff(
        &self,
        text_a: &str,
        text_b: &str,
        line_range: Option<(usize, usize)>,
    ) -> IncrementalDiffResult {
        match line_range {
            Some((start_line, end_line)) => {
                // Extract only the relevant lines
                let lines_a: Vec<&str> = text_a.lines().collect();
                let lines_b: Vec<&str> = text_b.lines().collect();

                let start = start_line.saturating_sub(2); // Include context
                let end = (end_line + 2).min(lines_a.len().max(lines_b.len()));

                let partial_a: String = lines_a
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &line)| {
                        if i >= start && i <= end {
                            Some(format!("{}\n", line))
                        } else {
                            None
                        }
                    })
                    .collect();

                let partial_b: String = lines_b
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &line)| {
                        if i >= start && i <= end {
                            Some(format!("{}\n", line))
                        } else {
                            None
                        }
                    })
                    .collect();

                self.compute_line_diff(&partial_a, &partial_b)
            }
            None => self.compute_line_diff(text_a, text_b),
        }
    }

    /// Fast check if two strings differ (without computing full diff).
    #[allow(dead_code)]
    pub fn quick_differ(&self, text_a: &str, text_b: &str) -> bool {
        text_a != text_b
    }
}
