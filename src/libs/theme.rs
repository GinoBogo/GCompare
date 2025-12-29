//! Loads and applies the application-wide CSS theme from style.css.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk};
use std::sync::Once;

/// Get the bundled CSS content (cached at compile time).
fn get_css_content() -> &'static str {
    include_str!("../style.css")
}

/// Initialize the application theme.
pub fn init() -> CssProvider {
    static INIT: Once = Once::new();

    let provider = CssProvider::new();
    provider.load_from_data(get_css_content());

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
    let mut css_content = get_css_content().to_string();

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
        (
            ".text-diff-remove",
            clean_color(&config.text_diff_remove_bg),
        ),
        (".text-diff-add", clean_color(&config.text_diff_add_bg)),
        (".text-diff-empty", clean_color(&config.text_diff_empty_bg)),
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

    // Apply the modified CSS
    provider.load_from_data(&css_content);
}

/// Get color property from a CSS class using the theme provider.
pub fn get_css_color(class_name: &str, property: &str) -> Option<gdk::RGBA> {
    parse_css_property(class_name, property)
}

/// Get foreground color from a CSS class.
pub fn get_color(class_name: &str) -> gdk::RGBA {
    get_css_color(class_name, "color").unwrap_or_else(|| {
        // Fallback to default gray
        gdk::RGBA::new(0.8, 0.8, 0.8, 1.0)
    })
}

/// Get background color from a CSS class.
pub fn get_background_color(class_name: &str) -> gdk::RGBA {
    get_css_color(class_name, "background-color").unwrap_or_else(|| {
        // Fallback to white
        gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)
    })
}

/// Parse any CSS property from the bundled CSS content.
fn parse_css_property(class_name: &str, property: &str) -> Option<gdk::RGBA> {
    let css_content = get_css_content();

    // Find the CSS class definition
    let class_pattern = format!(".{} {{", class_name);
    if let Some(class_start) = css_content.find(&class_pattern) {
        // Find the end of the class definition (closing brace)
        let class_content = &css_content[class_start + class_pattern.len()..];
        if let Some(class_end) = class_content.find('}') {
            let class_rules = &class_content[..class_end];

            // Parse the specific property
            for rule in class_rules.split(';') {
                let rule = rule.trim();
                if rule.starts_with(property) {
                    if let Some(value_part) = rule.split(':').nth(1) {
                        let value = value_part.trim();
                        return parse_color_value(value);
                    }
                }
            }
        }
    }

    None
}

/// Parse a color value (hex or rgba) into gdk::RGBA.
fn parse_color_value(value: &str) -> Option<gdk::RGBA> {
    // Parse hex color (e.g., "#ffffff")
    if value.starts_with('#') && value.len() == 7 {
        let r = u8::from_str_radix(&value[1..3], 16).ok()? as f64 / 255.0;
        let g = u8::from_str_radix(&value[3..5], 16).ok()? as f64 / 255.0;
        let b = u8::from_str_radix(&value[5..7], 16).ok()? as f64 / 255.0;
        let mut rgba = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
        rgba.set_red(r as f32);
        rgba.set_green(g as f32);
        rgba.set_blue(b as f32);
        rgba.set_alpha(1.0);
        return Some(rgba);
    } else if value.starts_with("rgba(") {
        // Parse rgba format: rgba(255, 255, 255, 0.8)
        let rgba_part = &value[5..value.len() - 1]; // Remove "rgba(" and ")"
        let parts: Vec<&str> = rgba_part.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse::<u8>().ok()? as f64 / 255.0;
            let g = parts[1].trim().parse::<u8>().ok()? as f64 / 255.0;
            let b = parts[2].trim().parse::<u8>().ok()? as f64 / 255.0;
            let a = parts[3].trim().parse::<f64>().ok()?;
            let mut rgba = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
            rgba.set_red(r as f32);
            rgba.set_green(g as f32);
            rgba.set_blue(b as f32);
            rgba.set_alpha(a as f32);
            return Some(rgba);
        }
    }

    None
}
