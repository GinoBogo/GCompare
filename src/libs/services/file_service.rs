//! File service for handling file operations.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{
    ApplicationWindow, ComboBoxText, Entry, FileChooserAction, FileChooserNative, ResponseType,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tokio::runtime::Runtime;

use crate::libs::widgets::gtextview::GTextView;

/// Service for handling file operations.
#[derive(Clone)]
pub struct FileService {
    active_dialog: Rc<RefCell<Option<FileChooserNative>>>,
}

impl FileService {
    /// Create a new file service.
    ///
    /// # Returns
    ///
    /// A new FileService instance with no active dialog
    pub fn new() -> Self {
        Self {
            active_dialog: Rc::new(RefCell::new(None)),
        }
    }

    /// Load file content from a given path into a GTextView using async approach.
    ///
    /// # Arguments
    ///
    /// * `text_view` - The GTextView widget to load the content into
    /// * `path_combo` - ComboBoxText containing the file path to load
    /// * `loading_flag` - Optional flag to indicate loading state
    ///                   (set to true during load, false after)
    ///
    /// # Returns
    ///
    /// * `(usize, usize)` - Tuple containing (bytes, lines) of the loaded file
    pub fn load_file_from_path(
        &self,
        text_view: &GTextView,
        path_combo: &ComboBoxText,
        loading_flag: Option<Rc<Cell<bool>>>,
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

        // Create tokio runtime for async operations
        let rt = Runtime::new().unwrap();

        // Load file asynchronously
        let result = rt.block_on(async { tokio::fs::read_to_string(&path_str).await });

        match result {
            Ok(content) => {
                let bytes = content.len();
                let lines = content.lines().count();

                // Set loading flag
                if let Some(flag) = &loading_flag {
                    flag.set(true);
                }

                // Update UI with loaded content
                text_view.set_text(&content);

                // Clear loading flag
                if let Some(flag) = &loading_flag {
                    flag.set(false);
                }

                (bytes, lines)
            }
            Err(_) => {
                // If reading fails (e.g., file deleted), clear view.
                text_view.clear();
                (0, 0)
            }
        }
    }

    /// Open file dialog and load selected file using async approach.
    ///
    /// # Arguments
    ///
    /// * `window` - The parent application window for the dialog
    /// * `text_view` - The TextView widget to load content into
    /// * `path_combo` - ComboBoxText containing the file path
    /// * `loading_flag` - Optional flag to indicate loading state
    /// * `on_load` - Optional callback function to execute after
    ///              successful load
    pub fn open_file_dialog<
        T: IsA<gtk::TextView> + IsA<gtk::Scrollable> + IsA<gtk::Widget> + Clone + 'static,
    >(
        &self,
        window: &ApplicationWindow,
        text_view: &T,
        path_combo: &ComboBoxText,
        loading_flag: Option<Rc<Cell<bool>>>,
        on_load: Option<Box<dyn Fn() + 'static>>,
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

        // Keep dialog alive by storing it in the service
        self.active_dialog.replace(Some(file_chooser.clone()));
        let active_dialog = self.active_dialog.clone();

        file_chooser.connect_response(move |dialog, response| {
            if response == ResponseType::Accept
                && let Some(file) = dialog.file()
                && let Some(path) = file.path()
            {
                // Create tokio runtime for async file reading
                let rt = Runtime::new().unwrap();
                let path_clone = path.clone();

                // Load file asynchronously
                let file_content_result =
                    rt.block_on(async { tokio::fs::read_to_string(&path_clone).await });

                if let Ok(file_content) = file_content_result {
                    // Set loading flag
                    if let Some(flag) = &loading_flag {
                        flag.set(true);
                    }

                    // Update UI with loaded content
                    text_view_clone.buffer().set_text(&file_content);

                    // Clear loading flag
                    if let Some(flag) = &loading_flag {
                        flag.set(false);
                    }

                    if let Some(entry) = path_combo_clone
                        .child()
                        .and_then(|child| child.downcast::<Entry>().ok())
                    {
                        entry.set_text(path.to_str().unwrap_or_default());
                    }
                    if let Some(callback) = &on_load {
                        callback();
                    }
                }
            }
            active_dialog.replace(None);
        });

        file_chooser.show();
    }

    /// Open save dialog and save TextView content to file using async approach.
    ///
    /// # Arguments
    ///
    /// * `window` - The parent application window for the dialog
    /// * `text_view` - The TextView widget containing content to save
    /// * `path_combo` - ComboBoxText containing the file path
    /// * `on_save` - Optional callback function to execute after
    ///              successful save
    pub fn save_file_dialog<T: IsA<gtk::TextView> + Clone + 'static>(
        &self,
        window: &ApplicationWindow,
        text_view: &T,
        path_combo: &ComboBoxText,
        on_save: Option<Box<dyn Fn() + 'static>>,
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
                if let Some(parent) = path.parent()
                    && parent.exists()
                {
                    let f = gtk::gio::File::for_path(parent);
                    file_chooser.set_current_folder(Some(&f)).ok();
                }
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    file_chooser.set_current_name(name);
                }
            }
        }

        let text_view_clone = text_view.clone();
        let path_combo_clone = path_combo.clone();

        // Keep the dialog alive by storing it in the service
        self.active_dialog.replace(Some(file_chooser.clone()));
        let active_dialog = self.active_dialog.clone();

        file_chooser.connect_response(move |dialog, response| {
            if response == ResponseType::Accept
                && let Some(file) = dialog.file()
                && let Some(path) = file.path()
            {
                let buffer = text_view_clone.buffer();
                let (start_iter, end_iter) = buffer.bounds();
                let file_content = buffer.text(&start_iter, &end_iter, true);

                // Create tokio runtime for async file writing
                let rt = Runtime::new().unwrap();
                let path_clone = path.clone();
                let content_clone = file_content.clone();

                // Save file asynchronously
                let save_result = rt.block_on(async {
                    tokio::fs::write(&path_clone, content_clone.as_str()).await
                });

                if save_result.is_ok()
                    && let Some(entry) = path_combo_clone
                        .child()
                        .and_then(|child| child.downcast::<Entry>().ok())
                {
                    entry.set_text(path.to_str().unwrap_or_default());
                    if let Some(callback) = &on_save {
                        callback();
                    }
                }
            }
            active_dialog.replace(None);
        });

        file_chooser.show();
    }
}
