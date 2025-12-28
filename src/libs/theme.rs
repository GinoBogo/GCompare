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
    // Load base CSS into memory
    let mut css_content = include_str!("../style.css").to_string();

    // Clean and validate color values
    let clean_color = |color: &str| {
        let cleaned = color
            .trim()
            .strip_prefix("#")
            .unwrap_or(color)
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect::<String>();

        // Ensure we have exactly 6 hex digits
        if cleaned.len() == 6 {
            cleaned
        } else if cleaned.len() == 3 {
            // Convert 3-digit hex to 6-digit (e.g., "f00" -> "ff0000")
            cleaned
                .chars()
                .flat_map(|c| std::iter::repeat(c).take(2))
                .collect()
        } else {
            // Fallback to default color if invalid
            "ffcccc".to_string()
        }
    };

    // Replace specific color values in the CSS
    let replacements = vec![
        (".diff-bg-remove", clean_color(&config.diff_remove_bg)),
        (".diff-bg-add", clean_color(&config.diff_add_bg)),
        (".diff-bg-empty", clean_color(&config.diff_empty_bg)),
    ];

    for (class_name, new_color) in replacements {
        // Find the CSS block for this class
        if let Some(start) = css_content.find(&format!("{} {{", class_name)) {
            // Find the end of the block
            if let Some(block_end) = css_content[start..].find("}") {
                let block_start = start;
                let block_end = start + block_end;

                // Find and replace the background-color in this block
                let block = &css_content[block_start..block_end];
                if let Some(bg_start) = block.find("background-color:") {
                    let bg_line_start = block_start + bg_start;
                    if let Some(bg_end) = css_content[bg_line_start..].find(";") {
                        let bg_line_end = bg_line_start + bg_end;

                        // Replace the color value
                        let before_bg = &css_content[..bg_line_start + "background-color:".len()];
                        let after_bg = &css_content[bg_line_end..];
                        css_content = format!("{} #{}{}", before_bg, new_color, after_bg);
                    }
                }
            }
        }
    }

    // Debug: print the generated CSS
    eprintln!("Updated CSS:\n{}", css_content);

    // Apply the modified CSS
    provider.load_from_data(&css_content);
}
