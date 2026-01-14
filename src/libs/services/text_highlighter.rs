//! Text highlighter service for applying diff highlighting to text buffers.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use crate::libs::services::color_parser::parse_color_with_fallback;
use crate::libs::state::AppConfig;
use gtk::prelude::*;

/// Service for applying and managing text highlighting.
#[derive(Clone)]
pub struct TextHighlighter;

impl TextHighlighter {
    /// Create a new TextHighlighter.
    pub fn new() -> Self {
        Self
    }

    /// Apply line-based highlighting to text buffers based on diff results.
    ///
    /// # Arguments
    ///
    /// * `buffer_a` - Text buffer for panel A
    /// * `buffer_b` - Text buffer for panel B
    /// * `changed_lines_a` - Line numbers with changes in panel A
    /// * `changed_lines_b` - Line numbers with changes in panel B
    /// * `empty_lines_a` - Empty line numbers in panel A
    /// * `empty_lines_b` - Empty line numbers in panel B
    /// * `config` - Application configuration with color settings
    pub fn apply_line_highlighting(
        &self,
        buffer_a: &gtk::TextBuffer,
        buffer_b: &gtk::TextBuffer,
        changed_lines_a: &[usize],
        changed_lines_b: &[usize],
        empty_lines_a: &[usize],
        empty_lines_b: &[usize],
        config: &AppConfig,
    ) {
        // Clear existing tags first
        self.clear_all_highlighting(buffer_a);
        self.clear_all_highlighting(buffer_b);

        // Apply tags for panel A (deletions)
        for &line_num in changed_lines_a {
            if let Some(start_iter) = buffer_a.iter_at_line(line_num as i32) {
                let mut end_iter = start_iter.clone();
                if end_iter.forward_line() {
                    self.apply_tag_to_range(
                        buffer_a,
                        &start_iter,
                        &end_iter,
                        "diff_remove",
                        config,
                    );
                }
            }
        }

        // Apply tags for panel B (additions)
        for &line_num in changed_lines_b {
            if let Some(start_iter) = buffer_b.iter_at_line(line_num as i32) {
                let mut end_iter = start_iter.clone();
                if end_iter.forward_line() {
                    self.apply_tag_to_range(buffer_b, &start_iter, &end_iter, "diff_add", config);
                }
            }
        }

        // Apply tags for empty lines in panel A
        for &line_num in empty_lines_a {
            if let Some(start_iter) = buffer_a.iter_at_line(line_num as i32) {
                let mut end_iter = start_iter.clone();
                if end_iter.forward_line() {
                    self.apply_tag_to_range(buffer_a, &start_iter, &end_iter, "diff_empty", config);
                }
            }
        }

        // Apply tags for empty lines in panel B
        for &line_num in empty_lines_b {
            if let Some(start_iter) = buffer_b.iter_at_line(line_num as i32) {
                let mut end_iter = start_iter.clone();
                if end_iter.forward_line() {
                    self.apply_tag_to_range(buffer_b, &start_iter, &end_iter, "diff_empty", config);
                }
            }
        }
    }

    /// Apply a specific tag to a text range.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Text buffer to apply tag to
    /// * `start` - Start iterator for the range
    /// * `end` - End iterator for the range
    /// * `tag_name` - Name of the tag to apply
    /// * `config` - Application configuration with color settings
    fn apply_tag_to_range(
        &self,
        buffer: &gtk::TextBuffer,
        start: &gtk::TextIter,
        end: &gtk::TextIter,
        tag_name: &str,
        config: &AppConfig,
    ) {
        // Ensure the tag exists
        self.ensure_tag_exists(buffer, tag_name, config);

        // Apply the tag
        buffer.apply_tag_by_name(tag_name, start, end);
    }

    /// Ensure a text tag exists in the buffer's tag table.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Text buffer to ensure tag exists in
    /// * `tag_name` - Name of the tag to ensure exists
    /// * `config` - Application configuration with color settings
    fn ensure_tag_exists(&self, buffer: &gtk::TextBuffer, tag_name: &str, config: &AppConfig) {
        if buffer.tag_table().lookup(tag_name).is_none() {
            let tag = gtk::TextTag::new(Some(tag_name));

            // Set colors based on tag type and config
            match tag_name {
                "diff_remove" => {
                    let bg_rgba =
                        parse_color_with_fallback(&config.text_diff_remove_bg, 255, 255, 255, 1.0);
                    let fg_rgba =
                        parse_color_with_fallback(&config.text_diff_remove_fg, 0, 0, 0, 1.0);
                    tag.set_background_rgba(Some(&bg_rgba));
                    tag.set_foreground_rgba(Some(&fg_rgba));
                }
                "diff_add" => {
                    let bg_rgba =
                        parse_color_with_fallback(&config.text_diff_add_bg, 255, 255, 255, 1.0);
                    let fg_rgba = parse_color_with_fallback(&config.text_diff_add_fg, 0, 0, 0, 1.0);
                    tag.set_background_rgba(Some(&bg_rgba));
                    tag.set_foreground_rgba(Some(&fg_rgba));
                }
                "diff_empty" => {
                    let bg_rgba =
                        parse_color_with_fallback(&config.text_diff_empty_bg, 255, 255, 255, 1.0);
                    let fg_rgba =
                        parse_color_with_fallback(&config.text_diff_empty_fg, 0, 0, 0, 1.0);
                    tag.set_background_rgba(Some(&bg_rgba));
                    tag.set_foreground_rgba(Some(&fg_rgba));
                }
                _ => {}
            }

            buffer.tag_table().add(&tag);
        }
    }

    /// Clear all diff highlighting from a buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Text buffer to clear highlighting from
    pub fn clear_all_highlighting(&self, buffer: &gtk::TextBuffer) {
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();

        buffer.remove_tag_by_name("diff_remove", &start_iter, &end_iter);
        buffer.remove_tag_by_name("diff_add", &start_iter, &end_iter);
        buffer.remove_tag_by_name("diff_empty", &start_iter, &end_iter);
    }

    /// Update highlighting for a single line (efficient for real-time).
    ///
    /// # Arguments
    ///
    /// * `buffer_a` - Text buffer for panel A
    /// * `buffer_b` - Text buffer for panel B
    /// * `line_number` - Line number to update
    /// * `config` - Application configuration with color settings
    #[allow(dead_code)]
    pub fn update_line_highlighting(
        &self,
        buffer_a: &gtk::TextBuffer,
        buffer_b: &gtk::TextBuffer,
        line_number: usize,
        config: &AppConfig,
    ) {
        // Get the line content from both buffers
        let line_a = Self::get_line_content(buffer_a, line_number);
        let line_b = Self::get_line_content(buffer_b, line_number);

        // Clear tags from this line in both buffers
        self.clear_line_highlighting(buffer_a, line_number);
        self.clear_line_highlighting(buffer_b, line_number);

        // Apply new tags if lines differ
        if line_a != line_b {
            if line_a.trim().is_empty() || line_b.trim().is_empty() {
                // Empty/whitespace difference
                if let Some(start_iter) = buffer_a.iter_at_line(line_number as i32) {
                    let mut end_iter = start_iter.clone();
                    if end_iter.forward_line() {
                        self.apply_tag_to_range(
                            buffer_a,
                            &start_iter,
                            &end_iter,
                            "diff_empty",
                            config,
                        );
                    }
                }
                if let Some(start_iter) = buffer_b.iter_at_line(line_number as i32) {
                    let mut end_iter = start_iter.clone();
                    if end_iter.forward_line() {
                        self.apply_tag_to_range(
                            buffer_b,
                            &start_iter,
                            &end_iter,
                            "diff_empty",
                            config,
                        );
                    }
                }
            } else {
                // Regular content difference
                if let Some(start_iter) = buffer_a.iter_at_line(line_number as i32) {
                    let mut end_iter = start_iter.clone();
                    if end_iter.forward_line() {
                        self.apply_tag_to_range(
                            buffer_a,
                            &start_iter,
                            &end_iter,
                            "diff_remove",
                            config,
                        );
                    }
                }
                if let Some(start_iter) = buffer_b.iter_at_line(line_number as i32) {
                    let mut end_iter = start_iter.clone();
                    if end_iter.forward_line() {
                        self.apply_tag_to_range(
                            buffer_b,
                            &start_iter,
                            &end_iter,
                            "diff_add",
                            config,
                        );
                    }
                }
            }
        }
    }

    /// Get content of a specific line.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Text buffer to get line content from
    /// * `line_number` - Line number to get content from
    ///
    /// # Returns
    ///
    /// * `String` - Content of the specified line
    #[allow(dead_code)]
    fn get_line_content(buffer: &gtk::TextBuffer, line_number: usize) -> String {
        if let Some(start_iter) = buffer.iter_at_line(line_number as i32) {
            let mut end_iter = start_iter.clone();
            if end_iter.forward_line() {
                return buffer.text(&start_iter, &end_iter, true).to_string();
            }
        }
        String::new()
    }

    /// Clear highlighting from a specific line.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Text buffer to clear highlighting from
    /// * `line_number` - Line number to clear highlighting from
    #[allow(dead_code)]
    fn clear_line_highlighting(&self, buffer: &gtk::TextBuffer, line_number: usize) {
        if let Some(start_iter) = buffer.iter_at_line(line_number as i32) {
            let mut end_iter = start_iter.clone();
            if end_iter.forward_line() {
                buffer.remove_tag_by_name("diff_remove", &start_iter, &end_iter);
                buffer.remove_tag_by_name("diff_add", &start_iter, &end_iter);
                buffer.remove_tag_by_name("diff_empty", &start_iter, &end_iter);
            }
        }
    }
}
