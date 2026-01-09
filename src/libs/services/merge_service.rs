//! Service for handling file merge operations.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use similar::TextDiff;

/// Strategy to use when merging files.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeStrategy {
    /// Use content from the first file (File A/Ours)
    AcceptOurs,
    /// Use content from the second file (File B/Theirs)
    AcceptTheirs,
    /// Include both versions
    Union,
    /// Generate standard conflict markers
    MarkConflicts,
}

/// Type of segment in the merged result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentType {
    Normal,
    FileA,
    FileB,
    Conflict,
}

/// Result of a merge operation including text and segments.
pub struct MergeResult {
    pub text: String,
    pub segments: Vec<(usize, usize, SegmentType)>,
}

#[derive(Clone)]
pub struct MergeService;

impl MergeService {
    /// Create a new MergeService.
    pub fn new() -> Self {
        Self
    }

    /// Merge two text strings based on the selected strategy.
    pub fn merge(&self, text_a: &str, text_b: &str, strategy: MergeStrategy) -> MergeResult {
        let diff = TextDiff::from_lines(text_a, text_b);

        struct MergeContext {
            result: String,
            segments: Vec<(usize, usize, SegmentType)>,
            current_char_count: usize,
        }

        impl MergeContext {
            fn append(&mut self, text: &str, seg_type: SegmentType) {
                if text.is_empty() {
                    return;
                }
                let start = self.current_char_count;
                self.result.push_str(text);
                let len = text.chars().count();
                self.current_char_count += len;
                self.segments.push((start, start + len, seg_type));
            }
        }

        let mut ctx = MergeContext {
            result: String::new(),
            segments: Vec::new(),
            current_char_count: 0,
        };

        for op in diff.ops() {
            match op.tag() {
                similar::DiffTag::Equal => {
                    // Content is the same, just append it
                    for change in diff.iter_changes(op) {
                        ctx.append(change.value(), SegmentType::Normal);
                    }
                }
                _ => {
                    // This is a change (Replace, Delete, Insert)
                    let mut content_a = String::new();
                    let mut content_b = String::new();

                    for change in diff.iter_changes(op) {
                        match change.tag() {
                            similar::ChangeTag::Delete => content_a.push_str(change.value()),
                            similar::ChangeTag::Insert => content_b.push_str(change.value()),
                            _ => {}
                        }
                    }

                    match strategy {
                        MergeStrategy::AcceptOurs => {
                            ctx.append(&content_a, SegmentType::FileA);
                        }
                        MergeStrategy::AcceptTheirs => {
                            ctx.append(&content_b, SegmentType::FileB);
                        }
                        MergeStrategy::Union => {
                            ctx.append(&content_a, SegmentType::FileA);
                            ctx.append(&content_b, SegmentType::FileB);
                        }
                        MergeStrategy::MarkConflicts => {
                            // Ensure we start on a new line if the previous content didn't end with one
                            if !ctx.result.is_empty() && !ctx.result.ends_with('\n') {
                                ctx.append("\n", SegmentType::Normal);
                            }

                            ctx.append("<<<<<<< File A\n", SegmentType::Conflict);
                            ctx.append(&content_a, SegmentType::FileA);

                            if !content_a.is_empty() && !content_a.ends_with('\n') {
                                ctx.append("\n", SegmentType::Normal);
                            }

                            ctx.append("=======\n", SegmentType::Conflict);
                            ctx.append(&content_b, SegmentType::FileB);

                            if !content_b.is_empty() && !content_b.ends_with('\n') {
                                ctx.append("\n", SegmentType::Normal);
                            }

                            ctx.append(">>>>>>> File B\n", SegmentType::Conflict);
                        }
                    }
                }
            }
        }

        MergeResult {
            text: ctx.result,
            segments: ctx.segments,
        }
    }
}
