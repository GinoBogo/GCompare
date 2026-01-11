//! Merge view window for displaying and saving merged content.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{
    Adjustment, ApplicationWindow, Box, FileChooserAction, FileChooserNative, Frame, Orientation,
    ResponseType, SizeGroup, SizeGroupMode, Window, glib,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::libs::services::color_parser::parse_color_with_fallback;
use crate::libs::services::config_service::ConfigService;
use crate::libs::services::merge_service::{
    MergeSegment, MergeService, MergeStrategy, SegmentType,
};
use crate::libs::widgets::gbutton::{ButtonTheme, GButton};
use crate::libs::widgets::gminimap::GMiniMap;
use crate::libs::widgets::gtextview::GTextView;

pub struct GMergeView {
    window: Window,
    #[allow(dead_code)]
    minimap: GMiniMap,
}

impl GMergeView {
    pub fn new(
        _parent: &ApplicationWindow,
        text_a: &str,
        text_b: &str,
        config_service: ConfigService,
    ) -> Self {
        let config = config_service.get_config();
        let window = Window::builder()
            .title("Merge Result")
            .default_width(config.merge_window_width)
            .default_height(config.merge_window_height)
            .build();

        // Create minimap
        let minimap = GMiniMap::new();

        let container = Box::new(Orientation::Vertical, 6);
        container.set_margin_top(6);
        container.set_margin_bottom(6);
        container.set_margin_start(6);
        container.set_margin_end(6);

        // Toolbar
        let toolbar = Box::new(Orientation::Horizontal, 6);

        let size_group = SizeGroup::new(SizeGroupMode::Horizontal);

        let btn_ours = GButton::new("Accept File A");
        btn_ours.set_tooltip_text(Some("Accept all changes from File A for the entire merge"));
        btn_ours.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_ours);

        let btn_theirs = GButton::new("Accept File B");
        btn_theirs.set_tooltip_text(Some("Accept all changes from File B for the entire merge"));
        btn_theirs.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_theirs);

        let btn_union = GButton::new("Union");
        btn_union.set_tooltip_text(Some("Include both versions in the merge result"));
        btn_union.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_union);

        let btn_conflicts = GButton::new("Mark Conflicts");
        btn_conflicts.set_tooltip_text(Some("Mark conflicts with standard conflict markers"));
        btn_conflicts.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_conflicts);

        let save_button = GButton::new("Save As...");
        save_button.set_tooltip_text(Some("Save the merged result to a file"));
        save_button.set_theme(ButtonTheme::Highlight);
        size_group.add_widget(&save_button);

        toolbar.append(&btn_ours);
        toolbar.append(&btn_theirs);
        toolbar.append(&btn_union);
        toolbar.append(&btn_conflicts);

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);

        toolbar.append(&save_button);

        container.append(&toolbar);

        // Text View with Minimap
        let text_view = GTextView::new();
        let text_frame = Frame::new(None);
        text_frame.set_vexpand(true);
        text_frame.set_child(Some(&text_view));

        // Create horizontal container for text frame and minimap
        let text_container = Box::new(Orientation::Horizontal, 6);
        text_container.set_vexpand(true);

        // Add text frame (expandable)
        text_frame.set_hexpand(true);
        text_container.append(&text_frame);

        // Create and add minimap frame (fixed width)
        let minimap_frame = Frame::new(None);
        minimap_frame.set_size_request(40, -1); // Fixed 40px width
        minimap_frame.set_child(Some(&minimap));
        text_container.append(&minimap_frame);

        container.append(&text_container);

        // Bottom Bar
        let bottom_bar = Box::new(Orientation::Horizontal, 6);
        bottom_bar.set_halign(gtk::Align::Center);
        let resolve_a_button = GButton::new("Accept Current A");
        resolve_a_button
            .set_tooltip_text(Some("Accept changes from File A for the current conflict"));
        resolve_a_button.set_theme(ButtonTheme::Action2);
        let resolve_b_button = GButton::new("Accept Current B");
        resolve_b_button
            .set_tooltip_text(Some("Accept changes from File B for the current conflict"));
        resolve_b_button.set_theme(ButtonTheme::Action1);
        let previous_button = GButton::new(" Prev ▲");
        previous_button.set_tooltip_text(Some("Navigate to previous conflict"));
        previous_button.set_theme(ButtonTheme::Highlight);
        let next_button = GButton::new(" Next ▼");
        next_button.set_tooltip_text(Some("Navigate to next conflict"));
        next_button.set_theme(ButtonTheme::Highlight);

        bottom_bar.append(&resolve_a_button);
        bottom_bar.append(&resolve_b_button);
        bottom_bar.append(&previous_button);
        bottom_bar.append(&next_button);
        container.append(&bottom_bar);

        // Setup tags
        let buffer = text_view.content_view().buffer();
        let create_tag = |name: &str, bg: &str, fg: &str| {
            if buffer.tag_table().lookup(name).is_none() {
                let tag = gtk::TextTag::new(Some(name));
                let bg_rgba = parse_color_with_fallback(bg, 255, 255, 255, 1.0);
                let fg_rgba = parse_color_with_fallback(fg, 0, 0, 0, 1.0);
                tag.set_background_rgba(Some(&bg_rgba));
                tag.set_foreground_rgba(Some(&fg_rgba));
                buffer.tag_table().add(&tag);
            }
        };

        create_tag(
            "diff_remove",
            &config.text_diff_remove_bg,
            &config.text_diff_remove_fg,
        );
        create_tag(
            "diff_add",
            &config.text_diff_add_bg,
            &config.text_diff_add_fg,
        );
        create_tag(
            "conflict",
            &config.merge_conflict_bg,
            &config.merge_conflict_fg,
        );

        window.set_child(Some(&container));

        // Logic
        let merge_service = MergeService::new();
        let text_a = text_a.to_string();
        let text_b = text_b.to_string();
        let segments_store = Rc::new(RefCell::new(Vec::<MergeSegment>::new()));

        // A closure to update the sensitivity of the resolve buttons based on
        // cursor position.
        let update_resolve_buttons_sensitivity = {
            let text_view = text_view.clone();
            let segments_store = segments_store.clone();
            let resolve_a_button = resolve_a_button.clone();
            let resolve_b_button = resolve_b_button.clone();

            Rc::new(move || {
                let buffer = text_view.content_view().buffer();
                if let Some(insert_mark) = buffer.mark("insert") {
                    let iter = buffer.iter_at_mark(&insert_mark);
                    let offset = iter.offset() as usize;

                    let segments = segments_store.borrow();
                    // A segment is considered active if the cursor is within
                    // its bounds (exclusive of the end).
                    let is_in_conflict = segments.iter().any(|s| {
                        offset >= s.start && offset < s.end && s.segment_type != SegmentType::Normal
                    });

                    resolve_a_button.set_sensitive(is_in_conflict);
                    resolve_a_button.set_opacity(if is_in_conflict { 1.0 } else { 0.5 });
                    resolve_b_button.set_sensitive(is_in_conflict);
                    resolve_b_button.set_opacity(if is_in_conflict { 1.0 } else { 0.5 });
                } else {
                    // Failsafe if the insert mark doesn't exist.
                    resolve_a_button.set_sensitive(false);
                    resolve_b_button.set_sensitive(false);
                    resolve_a_button.set_opacity(0.5);
                    resolve_b_button.set_opacity(0.5);
                }
            })
        };

        let update_view = {
            let text_view = text_view.clone();
            let merge_service = merge_service.clone();
            let text_a = text_a.clone();
            let text_b = text_b.clone();
            let btn_ours = btn_ours.clone();
            let btn_theirs = btn_theirs.clone();
            let btn_union = btn_union.clone();
            let btn_conflicts = btn_conflicts.clone();
            let segments_store = segments_store.clone();
            let update_sensitivity_on_view_update = update_resolve_buttons_sensitivity.clone();
            let minimap = minimap.clone();

            Rc::new(move |strategy: MergeStrategy| {
                btn_ours.set_sensitive(strategy != MergeStrategy::AcceptOurs);
                btn_theirs.set_sensitive(strategy != MergeStrategy::AcceptTheirs);
                btn_union.set_sensitive(strategy != MergeStrategy::Union);
                let is_conflicts = strategy == MergeStrategy::MarkConflicts;
                btn_conflicts.set_sensitive(!is_conflicts);
                btn_conflicts.set_opacity(if is_conflicts { 0.5 } else { 1.0 });

                let result = merge_service.merge(&text_a, &text_b, strategy);
                text_view.set_text(&result.text);
                *segments_store.borrow_mut() = result.segments.clone();

                // Update minimap with merged text segments
                let mut diff_lines_a = Vec::new();
                let mut diff_lines_b = Vec::new();

                // Extract diff lines from merge segments (merged text content)
                for segment in &result.segments {
                    let start_iter = buffer.iter_at_offset(segment.start as i32);
                    let end_iter = buffer.iter_at_offset(segment.end as i32);
                    let start_line = start_iter.line() as usize;
                    let end_line = end_iter.line() as usize;

                    for line in start_line..=end_line {
                        match segment.segment_type {
                            SegmentType::FileA => diff_lines_a.push(line),
                            SegmentType::FileB => diff_lines_b.push(line),
                            _ => {}
                        }
                    }
                }

                // Update minimap with merged text content
                minimap.set_all_diff_lines(
                    diff_lines_a, // File A content in merged text
                    diff_lines_b, // File B content in merged text
                    Vec::new(),   // No empty lines for simplicity
                    Vec::new(),   // No empty lines for simplicity
                );

                // Update cursor setup to use merged text line counts
                let buffer = text_view.content_view().buffer();
                let total_lines = buffer.line_count() as usize;

                // Calculate visible lines based on window height
                let visible_lines = if total_lines > 0 {
                    let window_height = text_view.content_view().allocation().height() as f64;
                    let line_height = 20.0;
                    (window_height / line_height) as usize
                } else {
                    35
                };

                minimap.update_text_info(
                    crate::libs::widgets::gminimap::PanelId::A,
                    0, // Start at top initially
                    total_lines,
                    visible_lines.max(5), // Ensure minimum cursor height
                );

                let buffer = text_view.content_view().buffer();
                for segment in result.segments {
                    let tag_name = match segment.segment_type {
                        SegmentType::FileA => "diff_remove",
                        SegmentType::FileB => "diff_add",
                        SegmentType::Conflict => "conflict",
                        _ => continue,
                    };

                    let start_iter = buffer.iter_at_offset(segment.start as i32);
                    let end_iter = buffer.iter_at_offset(segment.end as i32);
                    buffer.apply_tag_by_name(tag_name, &start_iter, &end_iter);
                }

                // After updating view and segments, update button sensitivity based on new cursor position.
                update_sensitivity_on_view_update();
            })
        };

        // Set up minimap cursor and scroll connection for merged text
        // Use default visible lines initially, will be updated when window is realized
        let initial_visible_lines = 35; // Default value

        minimap.update_text_info(
            crate::libs::widgets::gminimap::PanelId::A,
            0,                            // Start at top initially
            1,                            // Start with 1 line (will be updated after merge)
            initial_visible_lines.max(5), // Ensure minimum cursor height
        );

        // Update cursor when window is mapped (has proper allocation)
        let minimap_realize = minimap.clone();
        let text_view_realize = text_view.clone();

        window.connect_map(move |_| {
            // Calculate visible lines based on actual window height
            let visible_lines = {
                let window_height = text_view_realize.content_view().allocation().height() as f64;
                let line_height = 20.0;
                if window_height > 0.0 {
                    (window_height / line_height) as usize
                } else {
                    35
                }
            };

            // Get actual merged text line count
            let buffer = text_view_realize.content_view().buffer();
            let total_lines = buffer.line_count() as usize;

            minimap_realize.update_text_info(
                crate::libs::widgets::gminimap::PanelId::A,
                0,
                total_lines.max(1),
                visible_lines.max(5),
            );
        });

        // Connect text view scroll to minimap for cursor visibility
        if let Some(adj) = text_view.content_view().vadjustment() {
            let minimap_clone = minimap.clone();

            adj.connect_value_changed(move |adj| {
                let upper = adj.upper();
                let page_size = adj.page_size();
                let current_value = adj.value();

                // Ensure proper boundaries
                let max_value = (upper - page_size).max(0.0);
                let clamped_value = current_value.clamp(0.0, max_value);

                if max_value > 0.0 {
                    let ratio = clamped_value / max_value;
                    let upper_line = (ratio * upper) as usize;

                    minimap_clone.update_text_info(
                        crate::libs::widgets::gminimap::PanelId::A,
                        upper_line,
                        upper as usize,
                        page_size as usize,
                    );
                }
            });
        }

        // Connect minimap scroll-to signal to text view scrolling
        if let Some(adj) = text_view.content_view().vadjustment() {
            minimap.connect_local("scroll-to", false, move |values| {
                let ratio = values[1].get::<f64>().unwrap();

                // Helper to set adjustment based on ratio
                let set_adj_ratio = |adj: &Adjustment, r: f64| {
                    let upper = adj.upper();
                    let page_size = adj.page_size();
                    let max_value = (upper - page_size).max(0.0);
                    let new_value = (r * max_value).clamp(0.0, max_value);
                    adj.set_value(new_value);
                };

                set_adj_ratio(&adj, ratio);
                None
            });
        }

        // Add window resize handlers to update minimap cursor
        let minimap_clone_width = minimap.clone();
        let text_view_clone_width = text_view.clone();
        let text_a_clone_width = text_a.clone();
        let text_b_clone_width = text_b.clone();

        let minimap_clone_height = minimap.clone();
        let text_view_clone_height = text_view.clone();
        let text_a_clone_height = text_a.clone();
        let text_b_clone_height = text_b.clone();

        window.connect_default_width_notify(move |_| {
            // Update minimap when window width changes
            let total_lines_a = text_a_clone_width.lines().count();
            let total_lines_b = text_b_clone_width.lines().count();
            let total_lines = total_lines_a.max(total_lines_b);

            let window_height = text_view_clone_width.content_view().allocation().height() as f64;
            let line_height = 20.0;
            let visible_lines = if total_lines > 0 {
                (window_height / line_height) as usize
            } else {
                35
            };

            minimap_clone_width.update_text_info(
                crate::libs::widgets::gminimap::PanelId::A,
                0,
                total_lines,
                visible_lines.max(5),
            );
        });

        window.connect_default_height_notify(move |_| {
            // Update minimap when window height changes
            let total_lines_a = text_a_clone_height.lines().count();
            let total_lines_b = text_b_clone_height.lines().count();
            let total_lines = total_lines_a.max(total_lines_b);

            let window_height = text_view_clone_height.content_view().allocation().height() as f64;
            let line_height = 20.0;
            let visible_lines = if total_lines > 0 {
                (window_height / line_height) as usize
            } else {
                35
            };

            minimap_clone_height.update_text_info(
                crate::libs::widgets::gminimap::PanelId::A,
                0,
                total_lines,
                visible_lines.max(5),
            );
        });

        // Connect buttons
        let update_ours = update_view.clone();
        btn_ours.connect_clicked(move |_| update_ours(MergeStrategy::AcceptOurs));

        let update_theirs = update_view.clone();
        btn_theirs.connect_clicked(move |_| update_theirs(MergeStrategy::AcceptTheirs));

        let update_union = update_view.clone();
        btn_union.connect_clicked(move |_| update_union(MergeStrategy::Union));

        let update_conflicts = update_view.clone();
        btn_conflicts.connect_clicked(move |_| update_conflicts(MergeStrategy::MarkConflicts));

        // Update sensitivity on cursor move.
        let buffer_cursor = text_view.content_view().buffer();
        let update_sensitivity_on_cursor = update_resolve_buttons_sensitivity.clone();
        buffer_cursor.connect_mark_set(move |_, _, mark| {
            if mark.name().as_deref() == Some("insert") {
                update_sensitivity_on_cursor();
            }
        });

        // Resolve Current Logic
        let segments_store_resolve = segments_store.clone();
        let text_view_resolve = text_view.clone();
        let minimap_resolve = minimap.clone();

        let resolve_current = Rc::new(move |use_a: bool| {
            let buffer = text_view_resolve.content_view().buffer();
            let insert_mark = buffer.mark("insert").expect("Insert mark not found");
            let iter = buffer.iter_at_mark(&insert_mark);
            let offset = iter.offset() as usize;

            let mut segments = segments_store_resolve.borrow_mut();

            // Find segment at cursor
            let segment_info = segments
                .iter()
                .find(|s| offset >= s.start && offset < s.end)
                .map(|s| {
                    (
                        s.group_id,
                        s.content_a.clone(),
                        s.content_b.clone(),
                        s.segment_type,
                    )
                });

            if let Some((group_id, content_a, content_b, segment_type)) = segment_info {
                if segment_type == SegmentType::Normal {
                    return;
                }

                // Find range of the whole group
                let group_indices: Vec<usize> = segments
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.group_id == group_id)
                    .map(|(i, _)| i)
                    .collect();

                if group_indices.is_empty() {
                    return;
                }

                let first_idx = *group_indices.first().unwrap();
                let last_idx = *group_indices.last().unwrap();

                let start_offset = segments[first_idx].start;
                let end_offset = segments[last_idx].end;

                let new_content = if use_a { &content_a } else { &content_b };
                let new_len = new_content.chars().count();
                let old_len = end_offset - start_offset;
                let delta = new_len as isize - old_len as isize;

                // Update buffer
                let mut start_iter = buffer.iter_at_offset(start_offset as i32);
                let mut end_iter = buffer.iter_at_offset(end_offset as i32);
                buffer.delete(&mut start_iter, &mut end_iter);
                let mut insert_iter = buffer.iter_at_offset(start_offset as i32);
                buffer.insert(&mut insert_iter, new_content);

                // Update segments
                segments.drain(first_idx..=last_idx);

                let new_segment = MergeSegment {
                    start: start_offset,
                    end: start_offset + new_len,
                    segment_type: if use_a {
                        SegmentType::FileA
                    } else {
                        SegmentType::FileB
                    },
                    content_a,
                    content_b,
                    group_id,
                };
                segments.insert(first_idx, new_segment);

                // Shift subsequent segments
                for s in segments.iter_mut().skip(first_idx + 1) {
                    s.start = (s.start as isize + delta) as usize;
                    s.end = (s.end as isize + delta) as usize;
                }

                // Apply tag
                let tag_name = if use_a { "diff_remove" } else { "diff_add" };
                let start_iter = buffer.iter_at_offset(start_offset as i32);
                let end_iter = buffer.iter_at_offset((start_offset + new_len) as i32);
                buffer.apply_tag_by_name(tag_name, &start_iter, &end_iter);

                // Update minimap to reflect the resolved conflict
                let mut conflict_lines = Vec::new();
                let mut diff_lines_a = Vec::new();
                let mut diff_lines_b = Vec::new();

                for segment in segments.iter() {
                    // Convert byte offsets to actual line numbers
                    let start_iter = buffer.iter_at_offset(segment.start as i32);
                    let end_iter = buffer.iter_at_offset(segment.end as i32);
                    let start_line = start_iter.line() as usize;
                    let end_line = end_iter.line() as usize;

                    for line in start_line..=end_line {
                        match segment.segment_type {
                            SegmentType::FileA => diff_lines_a.push(line),
                            SegmentType::FileB => diff_lines_b.push(line),
                            SegmentType::Conflict => conflict_lines.push(line),
                            _ => {}
                        }
                    }
                }

                minimap_resolve.set_all_diff_lines(
                    diff_lines_a,
                    diff_lines_b,
                    conflict_lines.clone(), // empty_a - use conflict lines for now
                    Vec::new(),             // empty_b - no empty lines for merge view
                );
            }
        });

        let resolve_a = resolve_current.clone();
        resolve_a_button.connect_clicked(move |_| resolve_a(true));

        let resolve_b = resolve_current;
        resolve_b_button.connect_clicked(move |_| resolve_b(false));

        // Navigation functions
        let navigate_to_conflict = {
            let text_view = text_view.clone();
            let segments_store = segments_store.clone();
            move |direction: i32| {
                let buffer = text_view.content_view().buffer();
                let insert_mark = buffer.mark("insert").expect("Insert mark not found");
                let cursor_iter = buffer.iter_at_mark(&insert_mark);
                let cursor_offset = cursor_iter.offset() as usize;

                let segments = segments_store.borrow();
                let conflict_segments: Vec<_> = segments
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.segment_type == SegmentType::Conflict)
                    .collect();

                if conflict_segments.is_empty() {
                    return;
                }

                let current_pos = conflict_segments
                    .iter()
                    .position(|(_, s)| cursor_offset >= s.start && cursor_offset <= s.end)
                    .unwrap_or(0);

                let target_index = if direction > 0 {
                    if current_pos + 1 < conflict_segments.len() {
                        current_pos + 1
                    } else {
                        0 // Wrap to first
                    }
                } else {
                    if current_pos == 0 {
                        conflict_segments.len() - 1 // Wrap to last
                    } else {
                        current_pos - 1
                    }
                };

                if let Some((_, target_segment)) = conflict_segments.get(target_index) {
                    let target_iter = buffer.iter_at_offset(target_segment.start as i32);
                    buffer.place_cursor(&target_iter);

                    // Scroll the text view to show the conflict
                    let text_view_widget = text_view.content_view();
                    let mark = buffer.create_mark(None, &target_iter, true);
                    text_view_widget.scroll_to_mark(&mark, 0.25, false, 0.0, 0.0);
                    buffer.delete_mark(&mark);
                }
            }
        };

        let navigate_previous = navigate_to_conflict.clone();
        previous_button.connect_clicked(move |_| navigate_previous(-1));

        let navigate_next = navigate_to_conflict;
        next_button.connect_clicked(move |_| navigate_next(1));

        // Save button
        let window_clone = window.clone();
        let text_view_clone = text_view.clone();
        save_button.connect_clicked(move |_| {
            let file_chooser = FileChooserNative::builder()
                .title("Save Merged File")
                .transient_for(&window_clone)
                .action(FileChooserAction::Save)
                .accept_label("Save")
                .cancel_label("Cancel")
                .build();

            let text_view = text_view_clone.clone();
            let dialog_ref = Rc::new(RefCell::new(Some(file_chooser.clone())));
            let dialog_ref_clone = dialog_ref.clone();

            file_chooser.connect_response(move |dialog, response| {
                if response == ResponseType::Accept
                    && let Some(file) = dialog.file()
                    && let Some(path) = file.path()
                {
                    let buffer = text_view.content_view().buffer();
                    let (start, end) = buffer.bounds();
                    let content = buffer.text(&start, &end, true);
                    let _ = std::fs::write(path, content.as_str());
                }
                *dialog_ref_clone.borrow_mut() = None;
            });

            file_chooser.show();
        });

        // Initial state
        update_view(MergeStrategy::MarkConflicts);

        // Save window geometry when window is closed
        let window_clone = window.clone();
        let config_service_clone = config_service.clone();
        window.connect_close_request(move |_| {
            // Save current window geometry directly to in-memory config
            let current_width = window_clone.width();
            let current_height = window_clone.height();
            let default_width = window_clone.default_width();
            let default_height = window_clone.default_height();

            // Use default_size if current size is 0 or seems wrong
            let width = if current_width > 0 {
                current_width
            } else {
                default_width
            };
            let height = if current_height > 0 {
                current_height
            } else {
                default_height
            };
            let maximized = window_clone.is_maximized();

            // Update merge window geometry in memory and save to disk
            config_service_clone.update_merge_window_geometry(width, height, maximized);
            config_service_clone.save_config();

            glib::Propagation::Proceed
        });

        Self { window, minimap }
    }

    /// Display the merge view window to the user.
    pub fn show(&self) {
        self.window.present();
    }
}
