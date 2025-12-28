//! Loads and applies the application-wide CSS theme from style.css.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk};
use std::sync::Once;

/// Initialize the application theme.
pub fn init() -> CssProvider {
    static INIT: Once = Once::new();

    let provider = CssProvider::new();
    provider.load_from_data(include_str!("../style.css"));

    INIT.call_once(|| {
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    provider
}

/// Update a CSS provider with custom colors from config.
pub fn update_provider_with_config(provider: &CssProvider, config: &crate::libs::state::AppConfig) {
    // Get the base CSS content
    let base_css = include_str!("../style.css");

    // Create custom CSS with user-defined colors
    let custom_colors = format!(
        r#"
.diff-bg-remove {{
    background-color: {};
}}
.diff-bg-add {{
    background-color: {};
}}
.diff-bg-empty {{
    background-color: {};
}}
"#,
        config.diff_remove_bg, config.diff_add_bg, config.diff_empty_bg
    );

    // Combine base CSS with custom colors
    let merged_css = format!("{}\n{}", base_css, custom_colors);

    provider.load_from_data(&merged_css);
}

/// Get the CSS content as a string slice.
pub fn get_css_content() -> &'static str {
    include_str!("../style.css")
}
