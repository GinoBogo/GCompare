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
use crate::libs::services::merge_service::{MergeService, MergeStrategy, SegmentType};
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

        let update_view = {
            let text_view = text_view.clone();
            let merge_service = merge_service.clone();
            let text_a = text_a.clone();
            let text_b = text_b.clone();

            let btn_ours = btn_ours.clone();
            let btn_theirs = btn_theirs.clone();
            let btn_union = btn_union.clone();
            let btn_conflicts = btn_conflicts.clone();

            Rc::new(move |strategy: MergeStrategy| {
                btn_ours.set_sensitive(strategy != MergeStrategy::AcceptOurs);
                btn_theirs.set_sensitive(strategy != MergeStrategy::AcceptTheirs);
                btn_union.set_sensitive(strategy != MergeStrategy::Union);
                btn_conflicts.set_sensitive(strategy != MergeStrategy::MarkConflicts);

                let result = merge_service.merge(&text_a, &text_b, strategy);
                text_view.set_text(&result.text);

                let buffer = text_view.content_view().buffer();
                for (start, end, seg_type) in result.segments {
                    let tag_name = match seg_type {
                        SegmentType::FileA => "diff_remove",
                        SegmentType::FileB => "diff_add",
                        SegmentType::Conflict => "conflict",
                        _ => continue,
                    };

                    let start_iter = buffer.iter_at_offset(start as i32);
                    let end_iter = buffer.iter_at_offset(end as i32);
                    buffer.apply_tag_by_name(tag_name, &start_iter, &end_iter);
                }
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
