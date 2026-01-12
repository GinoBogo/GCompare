//! Custom GTextView widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::glib::translate::IntoGlib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{Adjustment, Box, GestureClick, PolicyType, ScrolledWindow, TextView, glib};
use once_cell::sync::OnceCell;
use std::cell::{Cell, RefCell};
use std::fmt::Write;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GTextView {
        // Gutter (line numbers) components
        pub gutter_view: OnceCell<TextView>,
        pub gutter_scrolled_window: OnceCell<ScrolledWindow>,

        // Content (text) components
        pub content_view: OnceCell<TextView>,
        pub content_scrolled_window: OnceCell<ScrolledWindow>,

        // State management
        pub line_count_cache: Cell<usize>,
        pub debounce_timeout: RefCell<Option<glib::SourceId>>,
        pub numbering_blocks: RefCell<Vec<(std::ops::Range<usize>, usize)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GTextView {
        const NAME: &'static str = "GTextView";
        type Type = super::GTextView;
        type ParentType = Box;
    }

    impl ObjectImpl for GTextView {
        /// Initialize widget when constructed.
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Configure main container
            obj.set_vexpand(true);
            obj.set_hexpand(true);
            obj.set_orientation(gtk::Orientation::Horizontal);
            obj.set_spacing(5);

            // Create gutter (line numbers) components

            let gutter_view = TextView::builder()
                .editable(false)
                .cursor_visible(false)
                .can_focus(false)
                .hexpand(false)
                .vexpand(true)
                .justification(gtk::Justification::Right)
                .monospace(true)
                .build();

            // Disable context menu on gutter
            let gesture = GestureClick::new();
            gesture.set_button(3); // Right click
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            gesture.connect_pressed(|gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
            });
            gutter_view.add_controller(gesture);

            gutter_view.add_css_class("gutter-numbers");

            let gutter_scrolled_window = ScrolledWindow::builder()
                .hexpand(false)
                .vexpand(true)
                .hscrollbar_policy(PolicyType::Never)
                .vscrollbar_policy(PolicyType::External)
                .child(&gutter_view)
                .build();

            // Create content (text) components

            let content_view = TextView::builder()
                .hexpand(true)
                .vexpand(true)
                .monospace(true)
                .build();

            let content_scrolled_window = ScrolledWindow::builder()
                .hexpand(true)
                .vexpand(true)
                .hscrollbar_policy(PolicyType::Automatic)
                .vscrollbar_policy(PolicyType::Automatic)
                .child(&content_view)
                .build();

            // Configure scrolling synchronization

            content_scrolled_window
                .vadjustment()
                .bind_property("value", &gutter_scrolled_window.vadjustment(), "value")
                .build();

            // Assemble widget hierarchy

            obj.append(&gutter_scrolled_window);
            obj.append(&content_scrolled_window);

            // Setup line number update mechanism

            let gutter_view_clone = gutter_view.clone();
            let gutter_scrolled_window_clone = gutter_scrolled_window.clone();
            let main_vadj = content_scrolled_window.vadjustment();

            // Store weak reference to self for closure
            let obj_weak = obj.downgrade();

            // Connect text change handler with debouncing
            content_view
                .buffer()
                .connect_changed(move |content_buffer| {
                    // Acquire GTextView instance
                    if let Some(obj) = obj_weak.upgrade() {
                        let imp = obj.imp();

                        // Cancel existing debounce timeout
                        if let Some(timeout_id) = imp.debounce_timeout.borrow_mut().take() {
                            timeout_id.remove();
                        }

                        // Clone captured variables for timeout closure
                        let gutter_view = gutter_view_clone.clone();
                        let gutter_scrolled_window = gutter_scrolled_window_clone.clone();
                        let main_vadj = main_vadj.clone();
                        let content_buffer = content_buffer.clone();
                        let obj_weak_timeout = obj_weak.clone();

                        // Schedule debounced update
                        let new_timeout_id = glib::timeout_add_local_once(
                            std::time::Duration::from_millis(50),
                            move || {
                                if let Some(obj) = obj_weak_timeout.upgrade() {
                                    let imp = obj.imp();
                                    // Clear the stored ID as it has now fired
                                    imp.debounce_timeout.borrow_mut().take();

                                    let new_line_count = content_buffer.line_count() as usize;
                                    let cached_count = imp.line_count_cache.get();

                                    // Skip update if line count unchanged and no blocks defined
                                    if new_line_count == cached_count
                                        && imp.numbering_blocks.borrow().is_empty()
                                    {
                                        return;
                                    }

                                    // Update line numbers
                                    update_line_numbers(
                                        &gutter_view,
                                        &gutter_scrolled_window,
                                        &main_vadj,
                                        new_line_count,
                                        cached_count,
                                        &imp.numbering_blocks.borrow(),
                                    );
                                    imp.line_count_cache.set(new_line_count);

                                    // Re-apply highlight as buffer might have been reset
                                    let insert = content_buffer.get_insert();
                                    let iter = content_buffer.iter_at_mark(&insert);
                                    highlight_gutter_line(&gutter_view, iter.line());
                                }
                            },
                        );

                        // Store new timeout ID
                        *imp.debounce_timeout.borrow_mut() = Some(new_timeout_id);
                    }
                });

            // Setup cursor movement handler for highlighting
            let gutter_view_weak = gutter_view.downgrade();
            content_view
                .buffer()
                .connect_mark_set(move |_, location, mark| {
                    if let Some(name) = mark.name() {
                        if name == "insert" {
                            if let Some(gutter_view) = gutter_view_weak.upgrade() {
                                highlight_gutter_line(&gutter_view, location.line());
                            }
                        }
                    }
                });

            // Store component references

            self.gutter_view.set(gutter_view).unwrap();
            self.gutter_scrolled_window
                .set(gutter_scrolled_window)
                .unwrap();

            self.content_view.set(content_view).unwrap();
            self.content_scrolled_window
                .set(content_scrolled_window)
                .unwrap();
        }

        /// Cleanup resources on widget destruction.
        fn dispose(&self) {
            // Cleanup pending debounce timeout
            if let Some(timeout_id) = self.debounce_timeout.borrow_mut().take() {
                timeout_id.remove();
            }
        }
    }

    impl WidgetImpl for GTextView {}
    impl BoxImpl for GTextView {}
}

glib::wrapper! {
    pub struct GTextView(ObjectSubclass<imp::GTextView>)
        @extends Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

/// Highlights the specified line in the gutter.
fn highlight_gutter_line(gutter_view: &TextView, line: i32) {
    let buffer = gutter_view.buffer();
    let tag_table = buffer.tag_table();

    // Create tag if it doesn't exist
    if tag_table.lookup("current-line").is_none() {
        let tag = gtk::TextTag::builder()
            .name("current-line")
            .weight(gtk::pango::Weight::Bold.into_glib())
            .build();
        tag_table.add(&tag);
    }

    // Remove existing highlight
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    buffer.remove_tag_by_name("current-line", &start, &end);

    // Apply new highlight
    if let Some(iter) = buffer.iter_at_line(line) {
        let mut end_iter = iter.clone();
        if !end_iter.ends_line() {
            end_iter.forward_to_line_end();
        }
        end_iter.forward_line();
        buffer.apply_tag_by_name("current-line", &iter, &end_iter);
    }
}

/// Efficiently updates line number display.
///
/// # Arguments
///
/// * `gutter_view` - TextView displaying line numbers.
/// * `gutter_scrolled_window` - ScrolledWindow containing the gutter view.
/// * `new_line_count` - Current number of lines in content.
/// * `old_line_count` - Previous number of lines in content.
fn update_line_numbers(
    gutter_view: &TextView,
    gutter_scrolled_window: &ScrolledWindow,
    main_vadj: &Adjustment,
    new_line_count: usize,
    old_line_count: usize,
    blocks: &[(std::ops::Range<usize>, usize)],
) {
    let gutter_buffer = gutter_view.buffer();

    // Handle empty content
    if new_line_count == 0 {
        gutter_buffer.set_text("");
        gutter_view.set_width_request(8 + 12);
        return;
    }

    // Calculate digit counts for width adjustment
    let new_digit_count = new_line_count.to_string().len();
    let old_digit_count = if old_line_count == 0 {
        1
    } else {
        old_line_count.to_string().len()
    };

    // Determine update strategy
    let has_blocks = !blocks.is_empty();
    let needs_full_update = new_line_count > old_line_count + 10
        || new_line_count < old_line_count.saturating_sub(10)
        || new_digit_count != old_digit_count
        || old_line_count == 0
        || has_blocks;

    // Full Update Strategy (large changes)

    if needs_full_update {
        let mut line_numbers = String::with_capacity(new_line_count * (new_digit_count + 1));
        if has_blocks {
            let mut block_iter = blocks.iter().peekable();
            let mut current_block = block_iter.next();

            for line_idx in 0..new_line_count {
                if let Some((range, start_num)) = current_block {
                    if range.contains(&line_idx) {
                        let num = start_num + (line_idx - range.start);
                        let _ = writeln!(line_numbers, "{}", num);
                    } else {
                        let _ = writeln!(line_numbers, "");
                    }

                    // If we just processed the last line of the block, advance
                    if line_idx + 1 == range.end {
                        current_block = block_iter.next();
                    }
                } else {
                    let _ = writeln!(line_numbers, "");
                }
            }
        } else {
            // Default continuous numbering
            for line in 1..=new_line_count {
                let _ = writeln!(line_numbers, "{}", line);
            }
        }

        // Remove trailing newline and set text
        if !line_numbers.is_empty() {
            line_numbers.pop(); // Remove final \n from writeln!
        }
        gutter_buffer.set_text(&line_numbers);
    }
    // Incremental Update Strategy (small changes)
    else {
        // Handle line addition
        if new_line_count > old_line_count {
            let mut additions =
                String::with_capacity((new_line_count - old_line_count) * (new_digit_count + 1));

            // Add separator if not first line
            if old_line_count > 0 {
                additions.push('\n');
            }

            for line in (old_line_count + 1)..=new_line_count {
                let _ = write!(additions, "{}", line);
                if line < new_line_count {
                    additions.push('\n');
                }
            }

            // Append new line numbers
            let mut end_iter = gutter_buffer.end_iter();
            gutter_buffer.insert(&mut end_iter, &additions);
        }
        // Handle line removal
        else if new_line_count < old_line_count {
            // Delete lines from the end
            if let Some(mut start_delete) = gutter_buffer.iter_at_line(new_line_count as i32) {
                // Move back to include the newline of the previous line
                if new_line_count > 0 && !start_delete.is_start() {
                    start_delete.backward_char();
                }

                let mut end_delete = gutter_buffer.end_iter();
                gutter_buffer.delete(&mut start_delete, &mut end_delete);
            }
        }
    }

    // Adjust gutter width if digit count changed

    if new_digit_count != old_digit_count {
        let new_width = (new_digit_count as i32 * 8) + 12;
        gutter_view.set_width_request(new_width);
    }

    // Force sync gutter scroll
    let gutter_vadj = gutter_scrolled_window.vadjustment();
    let target_val = main_vadj.value();

    if (gutter_vadj.value() - target_val).abs() > f64::EPSILON {
        gutter_vadj.set_value(target_val);
    }
}

impl GTextView {
    /// Create a new GTextView widget.
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    /// Get reference to gutter text view.
    pub fn gutter_view(&self) -> TextView {
        self.imp()
            .gutter_view
            .get()
            .expect("Gutter view not initialized")
            .clone()
    }

    /// Get reference to content text view.
    pub fn content_view(&self) -> TextView {
        self.imp()
            .content_view
            .get()
            .expect("Content view not initialized")
            .clone()
    }

    /// Set text content with immediate line number update.
    ///
    /// # Arguments
    ///
    /// * `text` - Text content to set.
    pub fn set_text(&self, text: &str) {
        let content_view = self.content_view();
        let gutter_view = self.gutter_view();
        let gutter_scrolled_window = self
            .imp()
            .gutter_scrolled_window
            .get()
            .expect("Scrolled window not initialized")
            .clone();
        let content_scrolled_window = self
            .imp()
            .content_scrolled_window
            .get()
            .expect("Scrolled window not initialized");
        let imp = self.imp();

        // Cancel pending debounce timeout
        if let Some(timeout_id) = imp.debounce_timeout.borrow_mut().take() {
            timeout_id.remove();
        }

        // Clear numbering blocks as text is changing
        self.clear_numbering_blocks();

        // Get current cursor position to preserve it
        let buffer = content_view.buffer();
        let cursor_iter = buffer.iter_at_offset(buffer.cursor_position());
        let cursor_line = cursor_iter.line();
        let cursor_line_offset = cursor_iter.line_offset();

        // Update content
        content_view.buffer().set_text(text);

        // Restore cursor position (if valid)
        let new_buffer = content_view.buffer();
        if let Some(new_iter) = new_buffer.iter_at_line(cursor_line) {
            let new_offset = new_iter.line_offset();
            if cursor_line_offset <= new_offset
                && let Some(restored_iter) =
                    new_buffer.iter_at_line_offset(cursor_line, cursor_line_offset)
            {
                new_buffer.place_cursor(&restored_iter);
            }
        }

        // Update line numbers immediately
        let new_line_count = content_view.buffer().line_count() as usize;
        update_line_numbers(
            &gutter_view,
            &gutter_scrolled_window,
            &content_scrolled_window.vadjustment(),
            new_line_count,
            imp.line_count_cache.get(),
            &imp.numbering_blocks.borrow(),
        );
        imp.line_count_cache.set(new_line_count);

        // Highlight current line
        let insert = content_view.buffer().get_insert();
        let iter = content_view.buffer().iter_at_mark(&insert);
        highlight_gutter_line(&gutter_view, iter.line());
    }

    /// Get current text content.
    pub fn get_text(&self) -> String {
        let content_view = self.content_view();
        let buffer = content_view.buffer();
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();

        buffer.text(&start_iter, &end_iter, true).to_string()
    }

    /// Clear text content.
    pub fn clear(&self) {
        self.set_text("");
    }

    /// Sets blocks of lines to be numbered. Each block restarts the count from 1.
    /// Lines outside of these blocks will not be numbered.
    pub fn set_numbering_blocks(&self, blocks: Vec<(std::ops::Range<usize>, usize)>) {
        let imp = self.imp();
        imp.numbering_blocks.replace(blocks);
        self.force_gutter_update();
    }

    /// Clears any numbering blocks, reverting the gutter to continuous numbering.
    pub fn clear_numbering_blocks(&self) {
        let imp = self.imp();
        let mut blocks = imp.numbering_blocks.borrow_mut();
        if !blocks.is_empty() {
            blocks.clear();
            self.force_gutter_update();
        }
    }

    /// Sets whether to show line numbers.
    pub fn set_show_line_numbers(&self, show: bool) {
        self.imp()
            .gutter_scrolled_window
            .get()
            .expect("Scrolled window not initialized")
            .set_visible(show);
    }

    /// Forces a full recalculation and redraw of the line number gutter.
    fn force_gutter_update(&self) {
        let imp = self.imp();
        let content_view = self.content_view();
        let gutter_view = self.gutter_view();
        let gutter_scrolled_window = imp
            .gutter_scrolled_window
            .get()
            .expect("Scrolled window not initialized");
        let content_scrolled_window = imp
            .content_scrolled_window
            .get()
            .expect("Scrolled window not initialized");
        let count = content_view.buffer().line_count() as usize;

        update_line_numbers(
            &gutter_view,
            gutter_scrolled_window,
            &content_scrolled_window.vadjustment(),
            count, // new line count
            0,     // Force full update
            &imp.numbering_blocks.borrow(),
        );
        // Update cache to prevent immediate redundant updates
        imp.line_count_cache.set(count);

        // Highlight current line
        let insert = content_view.buffer().get_insert();
        let iter = content_view.buffer().iter_at_mark(&insert);
        highlight_gutter_line(&gutter_view, iter.line());
    }
}
