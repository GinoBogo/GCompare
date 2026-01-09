//! Merge view window for displaying and saving merged content.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box, FileChooserAction, FileChooserNative, Frame, Orientation, ResponseType,
    SizeGroup, SizeGroupMode, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::libs::services::color_parser::parse_color_with_fallback;
use crate::libs::services::merge_service::{
    MergeSegment, MergeService, MergeStrategy, SegmentType,
};
use crate::libs::state::AppConfig;
use crate::libs::widgets::gbutton::{ButtonTheme, GButton};
use crate::libs::widgets::gtextview::GTextView;

pub struct GMergeView {
    window: Window,
}

impl GMergeView {
    pub fn new(
        _parent: &ApplicationWindow,
        text_a: &str,
        text_b: &str,
        config: &AppConfig,
    ) -> Self {
        let window = Window::builder()
            .title("Merge Result")
            .default_width(800)
            .default_height(600)
            .build();

        let container = Box::new(Orientation::Vertical, 6);
        container.set_margin_top(6);
        container.set_margin_bottom(6);
        container.set_margin_start(6);
        container.set_margin_end(6);

        // Toolbar
        let toolbar = Box::new(Orientation::Horizontal, 6);

        let size_group = SizeGroup::new(SizeGroupMode::Horizontal);

        let btn_ours = GButton::new("Accept File A");
        btn_ours.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_ours);

        let btn_theirs = GButton::new("Accept File B");
        btn_theirs.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_theirs);

        let btn_union = GButton::new("Union");
        btn_union.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_union);

        let btn_conflicts = GButton::new("Mark Conflicts");
        btn_conflicts.set_theme(ButtonTheme::Primary);
        size_group.add_widget(&btn_conflicts);

        let btn_save = GButton::new("Save As...");
        btn_save.set_theme(ButtonTheme::Highlight);
        size_group.add_widget(&btn_save);

        toolbar.append(&btn_ours);
        toolbar.append(&btn_theirs);
        toolbar.append(&btn_union);
        toolbar.append(&btn_conflicts);

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);

        toolbar.append(&btn_save);

        container.append(&toolbar);

        // Text View
        let text_view = GTextView::new();
        let frame = Frame::new(None);
        frame.set_vexpand(true);
        frame.set_child(Some(&text_view));
        container.append(&frame);

        // Bottom Bar
        let bottom_bar = Box::new(Orientation::Horizontal, 6);
        bottom_bar.set_halign(gtk::Align::Center);
        let btn_resolve_a = GButton::new("Accept Current A");
        btn_resolve_a.set_tooltip_text(Some("Accept changes from File A for the current conflict"));
        btn_resolve_a.set_theme(ButtonTheme::LightGreen);
        let btn_resolve_b = GButton::new("Accept Current B");
        btn_resolve_b.set_tooltip_text(Some("Accept changes from File B for the current conflict"));
        btn_resolve_b.set_theme(ButtonTheme::LightBlue);

        bottom_bar.append(&btn_resolve_a);
        bottom_bar.append(&btn_resolve_b);
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
        create_tag("conflict", "#ffcc00", "#000000");

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
            let btn_resolve_a = btn_resolve_a.clone();
            let btn_resolve_b = btn_resolve_b.clone();

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

                    btn_resolve_a.set_sensitive(is_in_conflict);
                    btn_resolve_b.set_sensitive(is_in_conflict);

                    let opacity = if is_in_conflict { 1.0 } else { 0.5 };
                    btn_resolve_a.set_opacity(opacity);
                    btn_resolve_b.set_opacity(opacity);
                } else {
                    // Failsafe if the insert mark doesn't exist.
                    btn_resolve_a.set_sensitive(false);
                    btn_resolve_b.set_sensitive(false);
                    btn_resolve_a.set_opacity(0.5);
                    btn_resolve_b.set_opacity(0.5);
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
            }
        });

        let resolve_a = resolve_current.clone();
        btn_resolve_a.connect_clicked(move |_| resolve_a(true));

        let resolve_b = resolve_current;
        btn_resolve_b.connect_clicked(move |_| resolve_b(false));

        // Save button
        let window_clone = window.clone();
        let text_view_clone = text_view.clone();
        btn_save.connect_clicked(move |_| {
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

        Self { window }
    }

    pub fn show(&self) {
        self.window.present();
    }
}
