//! File service for handling file operations.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{
    ApplicationWindow, ComboBoxText, Entry, FileChooserAction, FileChooserNative, ResponseType,
};

use crate::libs::widgets::gtextview::GTextView;

/// Service for handling file operations.
#[derive(Clone)]
pub struct FileService;

impl FileService {
    /// Create a new file service.
    pub fn new() -> Self {
        Self
    }

    /// Reload file content from a given path into a GTextView.
    pub fn reload_file_from_path(
        &self,
        text_view: &GTextView,
        path_combo: &ComboBoxText,
    ) -> (usize, usize) {
        let path_str = path_combo
            .child()
            .and_then(|c| c.downcast::<Entry>().ok())
            .map(|e| e.text().to_string())
            .unwrap_or_default();

        if path_str.is_empty() {
            text_view.clear();
            return (0, 0);
        }

        match std::fs::read_to_string(&path_str) {
            Ok(content) => {
                let bytes = content.len();
                let lines = content.lines().count();
                text_view.set_text(&content);
                (bytes, lines)
            }
            Err(_) => {
                // If reading fails (e.g., file deleted), clear view.
                text_view.clear();
                (0, 0)
            }
        }
    }

    /// Open file dialog and load selected file.
    pub fn open_file_dialog<
        T: IsA<gtk::TextView> + IsA<gtk::Scrollable> + IsA<gtk::Widget> + Clone + 'static,
    >(
        &self,
        window: &ApplicationWindow,
        text_view: &T,
        path_combo: &ComboBoxText,
    ) {
        let file_chooser = FileChooserNative::builder()
            .title("Open File")
            .transient_for(window)
            .action(FileChooserAction::Open)
            .accept_label("Open")
            .cancel_label("Cancel")
            .build();

        if let Some(entry) = path_combo.child().and_then(|c| c.downcast::<Entry>().ok()) {
            let path_str = entry.text().to_string();
            if !path_str.is_empty() {
                let path = std::path::Path::new(&path_str);
                if path.exists() {
                    let f = gtk::gio::File::for_path(path);
                    if path.is_file() {
                        file_chooser.set_file(&f).ok();
                    } else if path.is_dir() {
                        file_chooser.set_current_folder(Some(&f)).ok();
                    }
                }
            }
        }

        let text_view_clone = text_view.clone();
        let path_combo_clone = path_combo.clone();

        file_chooser.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        if let Ok(file_content) = std::fs::read_to_string(&path) {
                            text_view_clone.buffer().set_text(&file_content);

                            if let Some(entry) = path_combo_clone
                                .child()
                                .and_then(|child| child.downcast::<Entry>().ok())
                            {
                                entry.set_text(path.to_str().unwrap_or_default());
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        file_chooser.show();
    }

    /// Open save dialog and save TextView content to file.
    pub fn save_file_dialog<T: IsA<gtk::TextView> + Clone + 'static>(
        &self,
        window: &ApplicationWindow,
        text_view: &T,
        path_combo: &ComboBoxText,
    ) {
        let file_chooser = FileChooserNative::builder()
            .title("Save File")
            .transient_for(window)
            .action(FileChooserAction::Save)
            .accept_label("Save")
            .cancel_label("Cancel")
            .build();

        if let Some(entry) = path_combo.child().and_then(|c| c.downcast::<Entry>().ok()) {
            let path_str = entry.text().to_string();
            if !path_str.is_empty() {
                let path = std::path::Path::new(&path_str);
                if let Some(parent) = path.parent() {
                    if parent.exists() {
                        let f = gtk::gio::File::for_path(parent);
                        file_chooser.set_current_folder(Some(&f)).ok();
                    }
                }
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    file_chooser.set_current_name(name);
                }
            }
        }

        let text_view_clone = text_view.clone();
        let path_combo_clone = path_combo.clone();

        file_chooser.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let buffer = text_view_clone.buffer();
                        let (start_iter, end_iter) = buffer.bounds();
                        let file_content = buffer.text(&start_iter, &end_iter, true);

                        if std::fs::write(&path, file_content.as_str()).is_ok() {
                            if let Some(entry) = path_combo_clone
                                .child()
                                .and_then(|child| child.downcast::<Entry>().ok())
                            {
                                entry.set_text(path.to_str().unwrap_or_default());
                            }
                        }
                    }
                }
            }
            dialog.destroy();
        });

        file_chooser.show();
    }
}
