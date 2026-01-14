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

#[derive(Debug, Clone)]
pub struct MergeSegment {
    pub start: usize,
    pub end: usize,
    pub segment_type: SegmentType,
    pub content_a: String,
    pub content_b: String,
    pub group_id: usize,
}

/// Result of a merge operation including text and segments.
pub struct MergeResult {
    pub text: String,
    pub segments: Vec<MergeSegment>,
}

#[derive(Clone)]
pub struct MergeService;

impl MergeService {
    /// Create a new MergeService.
    pub fn new() -> Self {
        Self
    }

    /// Merge two text strings based on the selected strategy.
    ///
    /// # Arguments
    ///
    /// * `text_a` - Original text content
    /// * `text_b` - Modified text content
    /// * `strategy` - Merge strategy to apply
    ///
    /// # Returns
    ///
    /// * `MergeResult` - Result containing merged text and segment information
    pub fn merge(&self, text_a: &str, text_b: &str, strategy: MergeStrategy) -> MergeResult {
        let diff = TextDiff::from_lines(text_a, text_b);

        struct MergeContext {
            result: String,
            segments: Vec<MergeSegment>,
            current_char_count: usize,
        }

        impl MergeContext {
            fn append(
                &mut self,
                text: &str,
                seg_type: SegmentType,
                content_a: &str,
                content_b: &str,
                group_id: usize,
            ) {
                if text.is_empty() {
                    return;
                }
                let start = self.current_char_count;
                self.result.push_str(text);
                let len = text.chars().count();
                self.current_char_count += len;
                self.segments.push(MergeSegment {
                    start,
                    end: start + len,
                    segment_type: seg_type,
                    content_a: content_a.to_string(),
                    content_b: content_b.to_string(),
                    group_id,
                });
            }
        }

        let mut ctx = MergeContext {
            result: String::new(),
            segments: Vec::new(),
            current_char_count: 0,
        };

        let mut group_id = 0;

        for op in diff.ops() {
            group_id += 1;
            match op.tag() {
                similar::DiffTag::Equal => {
                    // Content is the same, just append it
                    for change in diff.iter_changes(op) {
                        let val = change.value();
                        ctx.append(val, SegmentType::Normal, val, val, group_id);
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
                            ctx.append(
                                &content_a,
                                SegmentType::FileA,
                                &content_a,
                                &content_b,
                                group_id,
                            );
                        }
                        MergeStrategy::AcceptTheirs => {
                            ctx.append(
                                &content_b,
                                SegmentType::FileB,
                                &content_a,
                                &content_b,
                                group_id,
                            );
                        }
                        MergeStrategy::Union => {
                            ctx.append(
                                &content_a,
                                SegmentType::FileA,
                                &content_a,
                                &content_b,
                                group_id,
                            );
                            ctx.append(
                                &content_b,
                                SegmentType::FileB,
                                &content_a,
                                &content_b,
                                group_id,
                            );
                        }
                        MergeStrategy::MarkConflicts => {
                            // Ensure we start on a new line if the previous content didn't end with one
                            if !ctx.result.is_empty() && !ctx.result.ends_with('\n') {
                                ctx.append("\n", SegmentType::Normal, "", "", group_id);
                            }

                            ctx.append(
                                "<<<<<<< File A\n",
                                SegmentType::Conflict,
                                &content_a,
                                &content_b,
                                group_id,
                            );
                            ctx.append(
                                &content_a,
                                SegmentType::FileA,
                                &content_a,
                                &content_b,
                                group_id,
                            );

                            if !content_a.is_empty() && !content_a.ends_with('\n') {
                                ctx.append(
                                    "\n",
                                    SegmentType::Normal,
                                    &content_a,
                                    &content_b,
                                    group_id,
                                );
                            }

                            ctx.append(
                                "=======\n",
                                SegmentType::Conflict,
                                &content_a,
                                &content_b,
                                group_id,
                            );
                            ctx.append(
                                &content_b,
                                SegmentType::FileB,
                                &content_a,
                                &content_b,
                                group_id,
                            );

                            if !content_b.is_empty() && !content_b.ends_with('\n') {
                                ctx.append(
                                    "\n",
                                    SegmentType::Normal,
                                    &content_a,
                                    &content_b,
                                    group_id,
                                );
                            }

                            ctx.append(
                                ">>>>>>> File B\n",
                                SegmentType::Conflict,
                                &content_a,
                                &content_b,
                                group_id,
                            );
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
