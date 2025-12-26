//! Loads and applies the application-wide CSS theme from style.css.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk};
use std::sync::Once;

/// Initialize the application theme.
pub fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let provider = CssProvider::new();
        provider.load_from_data(include_str!("../style.css"));

        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });
}
