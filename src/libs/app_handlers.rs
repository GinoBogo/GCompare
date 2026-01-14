//! Signal handlers for application interactions.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::ApplicationWindow;
use gtk::prelude::*;
use similar::ChangeTag;
use std::cell::Cell;
use std::rc::Rc;

use crate::libs::dialogs::goptionsdialog::GOptionsDialog;
use crate::libs::services::color_parser::parse_color_with_fallback;
use crate::libs::services::config_service::ConfigService;
use crate::libs::services::diff_service::DiffService;
use crate::libs::services::incremental_diff_service::IncrementalDiffService;
use crate::libs::services::text_highlighter::TextHighlighter;
use crate::libs::state::ApplicationState;
use crate::libs::theme;
use crate::libs::widgets::gbutton::GButton;
use crate::libs::widgets::gminimap::GMiniMap;
use crate::libs::widgets::gstatusbar::GStatusBar;
use crate::libs::widgets::gtextview::GTextView;

/// Setup the compare button interaction.
pub fn setup_compare_interaction(
    window: &ApplicationWindow,
    button: &GButton,
    panel_a: &GTextView,
    panel_b: &GTextView,
    panel_a_combo: &gtk::ComboBoxText,
    panel_b_combo: &gtk::ComboBoxText,
    minimap: &GMiniMap,
    status_bar: &GStatusBar,
    state: Rc<ApplicationState>,
    diff_service: DiffService,
    incremental_diff_service: IncrementalDiffService,
    text_highlighter: TextHighlighter,
    is_loading: Rc<Cell<bool>>,
) {
    let window = window.clone();
    let panel_a_text_view = panel_a.clone();
    let panel_b_text_view = panel_b.clone();
    let panel_a_combo = panel_a_combo.clone();
    let panel_b_combo = panel_b_combo.clone();
    let minimap = minimap.clone();
    let status_bar_compare = status_bar.clone();
    let state_for_colors = state;

    button.connect_clicked(move |_| {
        // Check if files are specified
        let get_path = |combo: &gtk::ComboBoxText| {
            combo
                .child()
                .and_then(|c| c.downcast::<gtk::Entry>().ok())
                .map(|e| e.text().to_string())
                .unwrap_or_default()
        };

        let path_a = get_path(&panel_a_combo);
        let path_b = get_path(&panel_b_combo);

        if path_a.trim().is_empty() && path_b.trim().is_empty() {
            let dialog = gtk::MessageDialog::builder()
                .transient_for(&window)
                .modal(true)
                .message_type(gtk::MessageType::Warning)
                .buttons(gtk::ButtonsType::Ok)
                .text("No Files Specified")
                .secondary_text("Please specify at least one file to compare.")
                .build();
            dialog.connect_response(|dlg, _| dlg.close());
            dialog.show();
            return;
        }

        let buffer_a = panel_a_text_view.content_view().buffer();
        let buffer_b = panel_b_text_view.content_view().buffer();

        // If auto-comparison is enabled, just refresh the display
        if state_for_colors.config().auto_compare {
            // Get current text content
            let (start_a, end_a) = buffer_a.bounds();
            let text_a = buffer_a.text(&start_a, &end_a, true);

            let (start_b, end_b) = buffer_b.bounds();
            let text_b = buffer_b.text(&start_b, &end_b, true);

            // Compute line differences
            let diff_result =
                incremental_diff_service.compute_line_diff(text_a.as_str(), text_b.as_str());

            // Apply highlighting
            let config = state_for_colors.config();
            let (empty_lines_a, empty_lines_b) = if config.ignore_whitespace {
                (Vec::new(), Vec::new())
            } else {
                (diff_result.empty_lines_a, diff_result.empty_lines_b)
            };

            text_highlighter.apply_line_highlighting(
                &buffer_a,
                &buffer_b,
                &diff_result.changed_lines_a,
                &diff_result.changed_lines_b,
                &empty_lines_a,
                &empty_lines_b,
                &config,
            );

            // Update diff map
            minimap.set_all_diff_lines(
                diff_result.changed_lines_a,
                diff_result.changed_lines_b,
                empty_lines_a,
                empty_lines_b,
            );

            // Update status bar
            status_bar_compare.update_status_from_buffers(&buffer_a, &buffer_b, &minimap);
            return;
        }

        // Original compare logic (for when auto-comparison is disabled)
        // Helper to get background color from config
        let get_theme_bg_color = |class_name: &str| {
            let config = &state_for_colors.config();
            let color = match class_name {
                "text-diff-remove" => &config.text_diff_remove_bg,
                "text-diff-add" => &config.text_diff_add_bg,
                "text-diff-empty" => &config.text_diff_empty_bg,
                _ => "#ffffff", // White default
            };

            // Use centralized color parsing
            parse_color_with_fallback(color, 255, 255, 255, 1.0)
        };

        // Helper to get foreground color from config
        let get_theme_fg_color = |class_name: &str| {
            let config = &state_for_colors.config();
            let color = match class_name {
                "text-diff-remove" => &config.text_diff_remove_fg,
                "text-diff-add" => &config.text_diff_add_fg,
                "text-diff-empty" => &config.text_diff_empty_fg,
                _ => "#000000", // Black default
            };

            // Use centralized color parsing
            parse_color_with_fallback(color, 0, 0, 0, 1.0)
        };

        // Create tags for highlighting if they don't exist
        let create_tag = |buffer: &gtk::TextBuffer, name: &str, css_class: &str| {
            if buffer.tag_table().lookup(name).is_none() {
                let tag = gtk::TextTag::new(Some(name));
                let bg_rgba = get_theme_bg_color(css_class);
                let fg_rgba = get_theme_fg_color(css_class);
                tag.set_background_rgba(Some(&bg_rgba));
                tag.set_foreground_rgba(Some(&fg_rgba));
                buffer.tag_table().add(&tag);
            }
        };

        create_tag(&buffer_a, "diff_remove", "text-diff-remove");
        create_tag(&buffer_b, "diff_add", "text-diff-add");
        create_tag(&buffer_a, "diff_empty", "text-diff-empty");
        create_tag(&buffer_b, "diff_empty", "text-diff-empty");

        // Get content
        let (start_a, end_a) = buffer_a.bounds();
        let text_a = buffer_a.text(&start_a, &end_a, true);

        let (start_b, end_b) = buffer_b.bounds();
        let text_b = buffer_b.text(&start_b, &end_b, true);

        // Compute Diff
        let changes = diff_service.compute_diff(text_a.as_str(), text_b.as_str());
        let ignore_whitespace = state_for_colors.config().ignore_whitespace;

        // Pre-compute all content and tag information using character positions
        let mut content_a = String::new();
        let mut content_b = String::new();
        let mut tag_ranges_a: Vec<(usize, usize, &str)> = Vec::new();
        let mut tag_ranges_b: Vec<(usize, usize, &str)> = Vec::new();
        let mut lines_a: Vec<usize> = Vec::new();
        let mut lines_b: Vec<usize> = Vec::new();
        let mut empty_lines_a: Vec<usize> = Vec::new();
        let mut empty_lines_b: Vec<usize> = Vec::new();
        let mut current_line_a = 0;
        let mut current_line_b = 0;

        // Build content and tag ranges in batch
        for change in &changes {
            match change.tag {
                ChangeTag::Equal => {
                    content_a.push_str(&change.content);
                    content_b.push_str(&change.content);
                    current_line_a += 1;
                    current_line_b += 1;
                }
                ChangeTag::Delete => {
                    let start_pos = content_a.chars().count();
                    content_a.push_str(&change.content);
                    let end_pos = content_a.chars().count();

                    // Check if content is empty or whitespace-only
                    let is_empty_or_whitespace = change.content.trim().is_empty();

                    if is_empty_or_whitespace {
                        if !ignore_whitespace {
                            empty_lines_a.push(current_line_a);
                            tag_ranges_a.push((start_pos, end_pos, "diff_empty"));
                        }
                    } else {
                        lines_a.push(current_line_a);
                        tag_ranges_a.push((start_pos, end_pos, "diff_remove"));
                    }

                    current_line_a += 1;
                }
                ChangeTag::Insert => {
                    let start_pos = content_b.chars().count();
                    content_b.push_str(&change.content);
                    let end_pos = content_b.chars().count();

                    // Check if content is empty or whitespace-only
                    let is_empty_or_whitespace = change.content.trim().is_empty();

                    if is_empty_or_whitespace {
                        if !ignore_whitespace {
                            empty_lines_b.push(current_line_b);
                            tag_ranges_b.push((start_pos, end_pos, "diff_empty"));
                        }
                    } else {
                        lines_b.push(current_line_b);
                        tag_ranges_b.push((start_pos, end_pos, "diff_add"));
                    }

                    current_line_b += 1;
                }
            }
        }

        // Batch update buffers with single operations
        is_loading.set(true);
        buffer_a.set_text(&content_a);
        buffer_b.set_text(&content_b);
        is_loading.set(false);

        // Apply all tags in batch using character positions
        for (start_char, end_char, tag_name) in tag_ranges_a {
            // Convert character positions to iterators
            let start_iter = buffer_a.iter_at_offset(start_char as i32);
            let end_iter = buffer_a.iter_at_offset(end_char as i32);
            buffer_a.apply_tag_by_name(tag_name, &start_iter, &end_iter);
        }

        for (start_char, end_char, tag_name) in tag_ranges_b {
            // Convert character positions to iterators
            let start_iter = buffer_b.iter_at_offset(start_char as i32);
            let end_iter = buffer_b.iter_at_offset(end_char as i32);
            buffer_b.apply_tag_by_name(tag_name, &start_iter, &end_iter);
        }

        minimap.set_all_diff_lines(lines_a, lines_b, empty_lines_a, empty_lines_b);

        // Update status bar with diff information
        status_bar_compare.update_status_from_buffers(&buffer_a, &buffer_b, &minimap);
    });
}

/// Setup the options button interaction.
pub fn setup_options_interaction(
    button: &GButton,
    window: &ApplicationWindow,
    state: Rc<ApplicationState>,
    config_service: ConfigService,
    panel_a: &GTextView,
    panel_b: &GTextView,
    css_provider: Rc<gtk::CssProvider>,
    minimap: &GMiniMap,
    sync_enabled: Rc<Cell<bool>>,
    control_panel: std::rc::Rc<crate::libs::ui::control_panel::ControlPanelWidget>,
    comparison_panels: std::rc::Rc<crate::libs::ui::comparison_panels::ComparisonPanelsWidget>,
) {
    let window_clone = window.clone();
    let state_clone = state;
    let config_service_clone = config_service;
    let panel_a_text_view = panel_a.clone();
    let panel_b_text_view = panel_b.clone();
    let css_provider_clone = css_provider;
    let minimap = minimap.clone();
    let sync_enabled = sync_enabled;

    button.connect_clicked(move |_| {
        // Get current config from memory (no file I/O)
        let current_config = config_service_clone.get_config();
        let dialog = GOptionsDialog::new(
            &window_clone,
            &current_config.font_family,
            current_config.font_size as f64,
            &current_config,
        );

        let state_clone_apply = state_clone.clone();
        let config_service_apply = config_service_clone.clone();
        let panel_a_text_view_apply = panel_a_text_view.clone();
        let panel_b_text_view_apply = panel_b_text_view.clone();
        let css_provider_apply = css_provider_clone.clone();
        let minimap_apply = minimap.clone();
        let sync_enabled_apply = sync_enabled.clone();
        let control_panel_apply = control_panel.clone();
        let comparison_panels_apply = comparison_panels.clone();
        dialog.show({
            let panel_a_text_view = panel_a_text_view.clone();
            let panel_b_text_view = panel_b_text_view.clone();
            move |result| {
                if let Some((font_family, font_size, color_config)) = result {
                    // Apply font changes to text views
                    panel_a_text_view_apply.set_font(&font_family, font_size);
                    panel_b_text_view_apply.set_font(&font_family, font_size);

                    // Update the in-memory state with new settings
                    let mut updated_config = state_clone_apply.config().clone();
                    updated_config.font_family = font_family.clone();
                    updated_config.font_size = font_size as i32;
                    updated_config.auto_compare = color_config.auto_compare;
                    updated_config.sync_scroll = color_config.sync_scroll;
                    updated_config.ignore_whitespace = color_config.ignore_whitespace;
                    updated_config.show_line_numbers = color_config.show_line_numbers;

                    // Update color settings
                    updated_config.text_diff_remove_bg = color_config.text_diff_remove_bg;
                    updated_config.text_diff_remove_fg = color_config.text_diff_remove_fg;
                    updated_config.text_diff_add_bg = color_config.text_diff_add_bg;
                    updated_config.text_diff_add_fg = color_config.text_diff_add_fg;
                    updated_config.text_diff_empty_bg = color_config.text_diff_empty_bg;
                    updated_config.text_diff_empty_fg = color_config.text_diff_empty_fg;
                    updated_config.merge_conflict_bg = color_config.merge_conflict_bg;
                    updated_config.merge_conflict_fg = color_config.merge_conflict_fg;
                    updated_config.gutter_numbers_bg = color_config.gutter_numbers_bg;
                    updated_config.gutter_numbers_fg = color_config.gutter_numbers_fg;
                    updated_config.minimap_bg = color_config.minimap_bg;
                    updated_config.minimap_fg = color_config.minimap_fg;
                    updated_config.minimap_diff_remove = color_config.minimap_diff_remove;
                    updated_config.minimap_diff_add = color_config.minimap_diff_add;
                    updated_config.minimap_diff_empty = color_config.minimap_diff_empty;
                    updated_config.minimap_cursor_bg = color_config.minimap_cursor_bg;

                    // Update the state in memory
                    state_clone_apply.update_config(updated_config.clone());

                    // Update runtime sync state
                    sync_enabled_apply.set(updated_config.sync_scroll);

                    // Update line numbers visibility
                    panel_a_text_view.set_show_line_numbers(updated_config.show_line_numbers);
                    panel_b_text_view.set_show_line_numbers(updated_config.show_line_numbers);

                    // Update theme with new colors
                    theme::update_provider_with_config(&css_provider_apply, &updated_config);

                    // Redraw minimap to pick up new cursor color
                    gtk::prelude::WidgetExt::queue_draw(&minimap_apply);

                    // Update text tag colors for real-time changes
                    let panel_a_text_view_apply = panel_a_text_view.clone();
                    let panel_b_text_view_apply = panel_b_text_view.clone();
                    let buffer_a = panel_a_text_view_apply.content_view().buffer();
                    let buffer_b = panel_b_text_view_apply.content_view().buffer();
                    theme::update_text_tag_colors(&[&buffer_a, &buffer_b], &updated_config);

                    // Update in-memory config and save to disk
                    config_service_apply.update_config(updated_config.clone());
                    config_service_apply.save_config();

                    // Update button colors
                    control_panel_apply.update_button_colors(&updated_config);
                    comparison_panels_apply.update_button_colors(&updated_config);
                }
            }
        });
    });
}

/// Setup navigation buttons (Previous/Next) interaction.
pub fn setup_navigation_interaction(
    window: &ApplicationWindow,
    prev_button: &GButton,
    next_button: &GButton,
    panel_a: &GTextView,
    minimap: &GMiniMap,
) {
    let panel_a_text_view_nav = panel_a.clone();
    let minimap_nav = minimap.clone();
    let window_prev = window.clone();

    prev_button.connect_clicked(move |_| {
        if !minimap_nav.has_differences() {
            let dialog = gtk::MessageDialog::builder()
                .transient_for(&window_prev)
                .modal(true)
                .message_type(gtk::MessageType::Info)
                .buttons(gtk::ButtonsType::Ok)
                .text("No Differences Found")
                .secondary_text("Please ensure a comparison has been performed.")
                .build();
            dialog.connect_response(|dlg, _| dlg.close());
            dialog.show();
            return;
        }
        // Get current line from panel A
        if let Some(adj) = panel_a_text_view_nav.content_view().vadjustment() {
            let y = adj.value().max(0.0) as i32;
            let (start_iter, _) = panel_a_text_view_nav.content_view().line_at_y(y);
            let current_line = start_iter.line() as usize;

            if let Some(target_line) = minimap_nav.previous_difference(current_line) {
                // Calculate scroll position for target line
                let buffer = panel_a_text_view_nav.content_view().buffer();
                if let Some(line_iter) = buffer.iter_at_line(target_line as i32) {
                    let line_y = panel_a_text_view_nav
                        .content_view()
                        .line_yrange(&line_iter)
                        .0 as f64;
                    adj.set_value(line_y);
                }
            }
        }
    });

    let panel_a_text_view_nav_next = panel_a.clone();
    let minimap_nav_next = minimap.clone();
    let window_next = window.clone();

    next_button.connect_clicked(move |_| {
        if !minimap_nav_next.has_differences() {
            let dialog = gtk::MessageDialog::builder()
                .transient_for(&window_next)
                .modal(true)
                .message_type(gtk::MessageType::Info)
                .buttons(gtk::ButtonsType::Ok)
                .text("No Differences Found")
                .secondary_text("Please ensure a comparison has been performed.")
                .build();
            dialog.connect_response(|dlg, _| dlg.close());
            dialog.show();
            return;
        }
        // Get current line from panel A
        if let Some(adj) = panel_a_text_view_nav_next.content_view().vadjustment() {
            let y = adj.value().max(0.0) as i32;
            let (start_iter, _) = panel_a_text_view_nav_next.content_view().line_at_y(y);
            let current_line = start_iter.line() as usize;

            if let Some(target_line) = minimap_nav_next.next_difference(current_line) {
                // Calculate scroll position for target line
                let buffer = panel_a_text_view_nav_next.content_view().buffer();
                if let Some(line_iter) = buffer.iter_at_line(target_line as i32) {
                    let line_y = panel_a_text_view_nav_next
                        .content_view()
                        .line_yrange(&line_iter)
                        .0 as f64;
                    adj.set_value(line_y);
                }
            }
        }
    });
}
