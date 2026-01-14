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
use crate::libs::widgets::gbutton::GButton;
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

        // Minimap
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
        btn_ours.set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);
        size_group.add_widget(&btn_ours);

        let btn_theirs = GButton::new("Accept File B");
        btn_theirs.set_tooltip_text(Some("Accept all changes from File B for the entire merge"));
        btn_theirs.set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);
        size_group.add_widget(&btn_theirs);

        let btn_union = GButton::new("Union");
        btn_union.set_tooltip_text(Some("Include both versions in the merge result"));
        btn_union.set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);
        size_group.add_widget(&btn_union);

        let btn_conflicts = GButton::new("Mark Conflicts");
        btn_conflicts.set_tooltip_text(Some("Mark conflicts with standard conflict markers"));
        btn_conflicts.set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);
        size_group.add_widget(&btn_conflicts);

        let save_button = GButton::new("Save As...");
        save_button.set_tooltip_text(Some("Save the merged result to a file"));
        save_button.set_custom_colors(&config.button_highlight_bg, &config.button_highlight_fg);
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

        // Text View
        let text_view = GTextView::new();
        text_view.set_show_line_numbers(config.show_line_numbers);
        text_view.set_font(&config.font_family, config.font_size as f64);

        let text_frame = Frame::new(None);
        text_frame.set_vexpand(true);
        text_frame.set_child(Some(&text_view));

        // Text container
        let text_container = Box::new(Orientation::Horizontal, 6);
        text_container.set_vexpand(true);

        // Text frame
        text_frame.set_hexpand(true);
        text_container.append(&text_frame);

        // Minimap frame
        let minimap_frame = Frame::new(None);
        minimap_frame.set_size_request(40, -1);
        minimap_frame.set_child(Some(&minimap));
        text_container.append(&minimap_frame);

        container.append(&text_container);

        // Bottom Bar
        let bottom_bar = Box::new(Orientation::Horizontal, 6);
        bottom_bar.set_halign(gtk::Align::Center);
        let resolve_a_button = GButton::new("Accept Block A");
        resolve_a_button
            .set_tooltip_text(Some("Accept changes from File A for the current conflict"));
        resolve_a_button
            .set_custom_colors(&config.text_diff_remove_bg, &config.text_diff_remove_fg);
        let resolve_b_button = GButton::new("Accept Block B");
        resolve_b_button
            .set_tooltip_text(Some("Accept changes from File B for the current conflict"));
        resolve_b_button.set_custom_colors(&config.text_diff_add_bg, &config.text_diff_add_fg);
        let previous_button = GButton::new(" Prev ▲");
        previous_button.set_tooltip_text(Some("Navigate to previous conflict"));
        previous_button.set_custom_colors(&config.button_highlight_bg, &config.button_highlight_fg);
        let next_button = GButton::new(" Next ▼");
        next_button.set_tooltip_text(Some("Navigate to next conflict"));
        next_button.set_custom_colors(&config.button_highlight_bg, &config.button_highlight_fg);

        bottom_bar.append(&resolve_a_button);
        bottom_bar.append(&resolve_b_button);
        bottom_bar.append(&previous_button);
        bottom_bar.append(&next_button);
        container.append(&bottom_bar);

        // Tags
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

        // Services
        let merge_service = MergeService::new();
        let text_a = text_a.to_string();
        let text_b = text_b.to_string();
        let segments_store = Rc::new(RefCell::new(Vec::<MergeSegment>::new()));

        // Update button sensitivity
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
                    // Check if cursor is in conflict segment
                    let is_in_conflict = segments.iter().any(|s| {
                        offset >= s.start && offset < s.end && s.segment_type != SegmentType::Normal
                    });

                    resolve_a_button.set_sensitive(is_in_conflict);
                    resolve_a_button.set_opacity(if is_in_conflict { 1.0 } else { 0.5 });
                    resolve_b_button.set_sensitive(is_in_conflict);
                    resolve_b_button.set_opacity(if is_in_conflict { 1.0 } else { 0.5 });
                } else {
                    // Failsafe
                    resolve_a_button.set_sensitive(false);
                    resolve_b_button.set_sensitive(false);
                    resolve_a_button.set_opacity(0.5);
                    resolve_b_button.set_opacity(0.5);
                }
            })
        };

        // Minimap helpers
        let update_minimap_content = {
            let minimap = minimap.clone();
            Rc::new(move |segments: &[MergeSegment], buffer: &gtk::TextBuffer| {
                let mut conflict_lines = Vec::new();
                let mut diff_lines_a = Vec::new();
                let mut diff_lines_b = Vec::new();

                for segment in segments {
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

                minimap.set_all_diff_lines(diff_lines_a, diff_lines_b, conflict_lines, Vec::new());
            })
        };

        let update_minimap_geometry = {
            let minimap = minimap.clone();
            let text_view = text_view.clone();
            Rc::new(move || {
                let content_view = text_view.content_view();
                let buffer = content_view.buffer();
                let total_lines = buffer.line_count() as usize;

                // Use adjustment for Y position to avoid lag
                let (y, height) = if let Some(adj) = content_view.vadjustment() {
                    (adj.value(), adj.page_size())
                } else {
                    let rect = content_view.visible_rect();
                    (rect.y() as f64, rect.height() as f64)
                };

                let y_int = y.max(0.0) as i32;
                let (start_iter, _) = content_view.line_at_y(y_int);
                let upper_line = start_iter.line() as usize;

                let (end_iter, _) = content_view.line_at_y(y_int + height as i32);
                let visible_lines = (end_iter.line() - start_iter.line()) as usize;

                minimap.update_text_info(
                    crate::libs::widgets::gminimap::PanelId::A,
                    upper_line,
                    total_lines.max(1),
                    visible_lines.max(5),
                );
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
            let update_minimap_content = update_minimap_content.clone();
            let update_minimap_geometry = update_minimap_geometry.clone();

            Rc::new(move |strategy: MergeStrategy| {
                // Update button state
                let update_btn_state = |btn: &GButton, is_active: bool| {
                    btn.set_sensitive(!is_active);
                    btn.set_opacity(if is_active { 0.5 } else { 1.0 });
                };
                update_btn_state(&btn_ours, strategy == MergeStrategy::AcceptOurs);
                update_btn_state(&btn_theirs, strategy == MergeStrategy::AcceptTheirs);
                update_btn_state(&btn_union, strategy == MergeStrategy::Union);
                update_btn_state(&btn_conflicts, strategy == MergeStrategy::MarkConflicts);

                let result = merge_service.merge(&text_a, &text_b, strategy);
                text_view.set_text(&result.text);
                *segments_store.borrow_mut() = result.segments.clone();

                if strategy == MergeStrategy::MarkConflicts {
                    let buffer = text_view.content_view().buffer();
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    let text = buffer.text(&start, &end, false);

                    let mut numbering_blocks = Vec::new();
                    let mut current_line_num = 1;
                    let mut saved_start_num = 1;
                    let mut max_num_reached = 1;

                    let mut current_block_start = 0;
                    let mut current_block_start_num = 1;
                    let mut in_block = true;

                    for (idx, line) in text.lines().enumerate() {
                        let is_marker = line.starts_with("<<<<<<<")
                            || line.starts_with("=======")
                            || line.starts_with(">>>>>>>");

                        if is_marker {
                            // Close block
                            if in_block {
                                if idx > current_block_start {
                                    numbering_blocks
                                        .push((current_block_start..idx, current_block_start_num));
                                }
                                in_block = false;
                            }

                            // Handle markers
                            if line.starts_with("<<<<<<<") {
                                saved_start_num = current_line_num;
                            } else if line.starts_with("=======") {
                                max_num_reached = std::cmp::max(max_num_reached, current_line_num);
                                current_line_num = saved_start_num;
                            } else if line.starts_with(">>>>>>>") {
                                max_num_reached = std::cmp::max(max_num_reached, current_line_num);
                                current_line_num = max_num_reached;
                            }
                        } else {
                            // Content
                            if !in_block {
                                current_block_start = idx;
                                current_block_start_num = current_line_num;
                                in_block = true;
                            }
                            current_line_num += 1;
                        }
                    }

                    // Close final block
                    if in_block {
                        let total_lines = buffer.line_count() as usize;
                        if total_lines > current_block_start {
                            numbering_blocks
                                .push((current_block_start..total_lines, current_block_start_num));
                        }
                    }

                    text_view.set_numbering_blocks(numbering_blocks);
                } else {
                    // Continuous numbering
                    text_view.clear_numbering_blocks();
                }

                let buffer = text_view.content_view().buffer();
                update_minimap_content(&result.segments, &buffer);
                update_minimap_geometry();

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

                // Update sensitivity
                update_sensitivity_on_view_update();
            })
        };

        // Minimap cursor setup
        let initial_visible_lines: usize = 35; // Default value

        minimap.update_text_info(
            crate::libs::widgets::gminimap::PanelId::A,
            0,                            // Start top
            1,                            // Initial lines
            initial_visible_lines.max(5), // Min height
        );

        // Update cursor on map
        let update_geom_map = update_minimap_geometry.clone();
        window.connect_map(move |_| {
            update_geom_map();
        });

        // Connect text view scroll to minimap for cursor visibility
        if let Some(adj) = text_view.content_view().vadjustment() {
            let update_geom_scroll = update_minimap_geometry.clone();
            adj.connect_value_changed(move |_| {
                // This is more accurate than calculating from adjustment values
                update_geom_scroll();
            });
        }

        // Sync minimap to scroll
        if let Some(adj) = text_view.content_view().vadjustment() {
            minimap.connect_local("scroll-to", false, move |values| {
                let ratio = values[1].get::<f64>().unwrap();

                // Set adjustment
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

        // Resize handlers
        let update_geom_width = update_minimap_geometry.clone();
        window.connect_default_width_notify(move |_| {
            update_geom_width();
        });

        let update_geom_height = update_minimap_geometry.clone();
        window.connect_default_height_notify(move |_| {
            update_geom_height();
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

        // Cursor move sensitivity
        let buffer_cursor = text_view.content_view().buffer();
        let update_sensitivity_on_cursor = update_resolve_buttons_sensitivity.clone();
        buffer_cursor.connect_mark_set(move |_, _, mark| {
            if mark.name().as_deref() == Some("insert") {
                update_sensitivity_on_cursor();
            }
        });

        // Resolve logic
        let segments_store_resolve = segments_store.clone();
        let text_view_resolve = text_view.clone();
        let update_minimap_content_resolve = update_minimap_content.clone();
        let update_minimap_geometry_resolve = update_minimap_geometry.clone();

        let resolve_current = Rc::new(move |use_a: bool| {
            let buffer = text_view_resolve.content_view().buffer();
            let insert_mark = buffer.mark("insert").expect("Insert mark not found");
            let iter = buffer.iter_at_mark(&insert_mark);
            let offset = iter.offset() as usize;

            let mut segments = segments_store_resolve.borrow_mut();

            // Find segment
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

                // Atomic update to prevent tag corruption
                buffer.begin_user_action();

                // Find group range
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
                    segment_type: SegmentType::Normal,
                    content_a,
                    content_b,
                    group_id,
                };
                segments.insert(first_idx, new_segment);

                // Shift segments
                for s in segments.iter_mut().skip(first_idx + 1) {
                    s.start = (s.start as isize + delta) as usize;
                    s.end = (s.end as isize + delta) as usize;
                }

                // Repaint tags
                let (start_all, end_all) = buffer.bounds();
                buffer.remove_tag_by_name("diff_remove", &start_all, &end_all);
                buffer.remove_tag_by_name("diff_add", &start_all, &end_all);
                buffer.remove_tag_by_name("conflict", &start_all, &end_all);

                // Apply tags
                for segment in segments.iter() {
                    let tag_name = match segment.segment_type {
                        SegmentType::FileA => "diff_remove",
                        SegmentType::FileB => "diff_add",
                        SegmentType::Conflict => "conflict",
                        SegmentType::Normal => continue,
                    };
                    let start_iter = buffer.iter_at_offset(segment.start as i32);
                    let end_iter = buffer.iter_at_offset(segment.end as i32);
                    buffer.apply_tag_by_name(tag_name, &start_iter, &end_iter);
                }

                buffer.end_user_action();

                // Update minimap
                update_minimap_content_resolve(&segments, &buffer);
                update_minimap_geometry_resolve();
            }
        });

        let resolve_a = resolve_current.clone();
        resolve_a_button.connect_clicked(move |_| resolve_a(true));

        let resolve_b = resolve_current;
        resolve_b_button.connect_clicked(move |_| resolve_b(false));

        // Navigation
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

                    // Scroll to conflict
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

        // Save
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

        // Save geometry on close
        let window_clone = window.clone();
        let config_service_clone = config_service.clone();
        window.connect_close_request(move |_| {
            // Get geometry
            let current_width = window_clone.width();
            let current_height = window_clone.height();
            let default_width = window_clone.default_width();
            let default_height = window_clone.default_height();

            // Validate size
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

            // Save config
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
