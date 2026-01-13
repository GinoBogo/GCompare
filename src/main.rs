//! Main entry point for GCompare application.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

#![windows_subsystem = "windows"]

use gtk::Application;
use gtk::prelude::*;

mod libs;
use libs::AppController;

/// Main entry point for GCompare application.
fn main() {
    let app = Application::builder()
        .application_id("com.gcompare.desktop")
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    // Handle normal activation (no files)
    app.connect_activate(|app| {
        build_ui(app, None, None);
    });

    // Handle file opening (when files are passed as arguments)
    app.connect_open(move |app, files, _hint| {
        // Extract file paths from gio::File objects
        let paths: Vec<String> = files
            .iter()
            .filter_map(|file| file.path())
            .filter_map(|path| path.to_str().map(|s| s.to_string()))
            .collect();

        // Get the first two files as A and B
        let file_a = paths.first().cloned();
        let file_b = paths.get(1).cloned();

        build_ui(app, file_a, file_b);
    });

    app.run();
}

/// Builds main application user interface using application controller.
///
/// # Arguments
///
/// * `app` - Application instance.
/// * `file_a_path` - Optional path to file A.
/// * `file_b_path` - Optional path to file B.
fn build_ui(app: &Application, file_a_path: Option<String>, file_b_path: Option<String>) {
    let mut app_controller = AppController::new();
    app_controller.initialize_ui(app, file_a_path, file_b_path);
    app_controller.show();
}
