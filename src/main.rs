//! Main entry point for GCompare application.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::Application;

mod libs;
use libs::AppController;

/// Main entry point for GCompare application.
fn main() {
    let app = Application::builder()
        .application_id("com.gcompare.desktop")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

/// Builds main application user interface using application controller.
///
/// # Arguments
///
/// * `app` - Application instance.
fn build_ui(app: &Application) {
    let mut app_controller = AppController::new();
    app_controller.initialize_ui(app);
    app_controller.show();
}
