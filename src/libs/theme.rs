//! Loads and applies the application-wide CSS theme from style.css.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, gdk, prelude::*};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Once;

use crate::libs::services::color_parser::parse_color_with_fallback;

/// Get the bundled CSS content (cached at compile time).
fn get_css_content() -> &'static str {
    include_str!("../style.css")
}

/// Parse CSS into a structured representation.
#[derive(Debug, Clone)]
struct CssRule {
    selector: String,
    properties: HashMap<String, String>,
}

impl CssRule {
    /// Create a new CSS rule with the given selector.
    ///
    /// # Arguments
    ///
    /// * `selector` - CSS selector string for the rule
    ///
    /// # Returns
    ///
    /// Returns a new `CssRule` instance with the specified selector
    fn new(selector: String) -> Self {
        Self {
            selector,
            properties: HashMap::new(),
        }
    }

    /// Convert the CSS rule to a formatted CSS string.
    ///
    /// # Returns
    ///
    /// Returns a string containing the CSS rule in proper CSS format
    fn to_css_string(&self) -> String {
        let mut result = format!("{} {{\n", self.selector);
        for (prop, value) in &self.properties {
            if value.len() == 8 && value.starts_with('#') {
                // 8-digit hex with alpha channel
                result.push_str(&format!("  {}: {};\n", prop, value));
            } else {
                // Regular 6-digit hex color
                result.push_str(&format!("  {}: {};\n", prop, value));
            }
        }
        result.push_str("}\n");
        result
    }
}

/// Parse CSS content into structured rules.
///
/// # Arguments
///
/// * `css_content` - Raw CSS content string to parse
///
/// # Returns
///
/// Returns a vector of `CssRule` instances representing the parsed CSS
fn parse_css_rules(css_content: &str) -> Vec<CssRule> {
    let mut rules = Vec::new();

    // Regex to match CSS rules: .selector { property: value; }
    let rule_regex = Regex::new(r"([^{]+)\s*\{([^}]+)\}").unwrap();
    let prop_regex = Regex::new(r"([a-zA-Z-]+)\s*:\s*([^;]+)").unwrap();

    for captures in rule_regex.captures_iter(css_content) {
        let selector = captures[1].trim().to_string();
        let properties_content = &captures[2];

        let mut rule = CssRule::new(selector);

        // Parse individual properties
        for prop_captures in prop_regex.captures_iter(properties_content) {
            let prop_name = prop_captures[1].trim().to_string();
            let prop_value = prop_captures[2].trim().to_string();
            rule.properties.insert(prop_name, prop_value);
        }

        rules.push(rule);
    }

    rules
}

/// Initialize the application theme.
///
/// # Returns
///
/// Returns a configured `CssProvider` with the default theme loaded
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
///
/// # Arguments
///
/// * `provider` - CSS provider to update with new colors
/// * `config` - Application configuration containing color settings
///
/// # Returns
///
/// Updates the provider in-place with new color values
pub fn update_provider_with_config(provider: &CssProvider, config: &crate::libs::state::AppConfig) {
    // Parse CSS into structured rules
    let mut rules = parse_css_rules(get_css_content());

    // Helper function to clean and validate color values
    let clean_color = |color: &str| {
        let cleaned = color
            .trim()
            .strip_prefix("#")
            .unwrap_or(color)
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect::<String>();

        // Ensure we have exactly 6 or 8 hex digits
        if cleaned.len() == 6 {
            cleaned
        } else if cleaned.len() == 3 {
            // Convert 3-digit hex to 6-digit (e.g., "f00" -> "ff0000")
            cleaned
                .chars()
                .flat_map(|c| std::iter::repeat_n(c, 2))
                .collect()
        } else if cleaned.len() == 8 {
            // 8-digit hex with alpha channel (e.g., "#00000008")
            cleaned
        } else {
            // Fallback to default color if invalid
            "ffcccc".to_string()
        }
    };

    // Define color mappings
    let color_mappings = [
        // Button colors
        (
            ".gbutton-default",
            "background-color",
            &config.button_default_bg,
        ),
        (".gbutton-default", "color", &config.button_default_fg),
        (
            ".gbutton-primary",
            "background-color",
            &config.button_primary_bg,
        ),
        (".gbutton-primary", "color", &config.button_primary_fg),
        (
            ".gbutton-secondary",
            "background-color",
            &config.button_secondary_bg,
        ),
        (".gbutton-secondary", "color", &config.button_secondary_fg),
        (
            ".gbutton-highlight",
            "background-color",
            &config.button_highlight_bg,
        ),
        (".gbutton-highlight", "color", &config.button_highlight_fg),
        // Text diff colors
        (
            ".text-diff-remove",
            "background-color",
            &config.text_diff_remove_bg,
        ),
        (".text-diff-remove", "color", &config.text_diff_remove_fg),
        (
            ".text-diff-add",
            "background-color",
            &config.text_diff_add_bg,
        ),
        (".text-diff-add", "color", &config.text_diff_add_fg),
        (
            ".text-diff-empty",
            "background-color",
            &config.text_diff_empty_bg,
        ),
        (".text-diff-empty", "color", &config.text_diff_empty_fg),
        // Gutter numbers colors
        (
            ".gutter-numbers",
            "background-color",
            &config.gutter_numbers_bg,
        ),
        (".gutter-numbers", "color", &config.gutter_numbers_fg),
        // Minimap colors
        (".minimap", "background-color", &config.minimap_bg),
        (".minimap", "color", &config.minimap_fg),
        (".minimap-diff-remove", "color", &config.minimap_diff_remove),
        (".minimap-diff-add", "color", &config.minimap_diff_add),
        (".minimap-diff-empty", "color", &config.minimap_diff_empty),
        (
            ".minimap-cursor",
            "background-color",
            &config.minimap_cursor_bg,
        ),
        // Merge conflict colors
        (
            ".merge-conflict",
            "background-color",
            &config.merge_conflict_bg,
        ),
        (".merge-conflict", "color", &config.merge_conflict_fg),
    ];

    // Update CSS properties using the structured tree
    for rule in &mut rules {
        for (class_selector, property, config_value) in &color_mappings {
            // Use exact match instead of contains to avoid partial matches
            if rule.selector.trim() == class_selector.trim() {
                // Apply hex format for all colors (including 8-digit with alpha)
                let cleaned_color = clean_color(config_value);
                rule.properties
                    .insert(property.to_string(), format!("#{}", cleaned_color));
            }
        }
    }

    // Generate CSS string from the modified rules
    let mut css_string = String::new();
    for rule in &rules {
        css_string.push_str(&rule.to_css_string());
    }

    // Update CSS provider with new configuration
    provider.load_from_data(&css_string);
}

/// Update text tag colors for real-time theme changes.
///
/// # Arguments
///
/// * `text_buffers` - Slice of text buffer references to update
/// * `config` - Application configuration with color settings
///
/// # Returns
///
/// Updates the text buffers' tag colors in-place
pub fn update_text_tag_colors(
    text_buffers: &[&gtk::TextBuffer],
    config: &crate::libs::state::AppConfig,
) {
    let bg_color_mappings = [
        ("diff_remove", &config.text_diff_remove_bg),
        ("diff_add", &config.text_diff_add_bg),
        ("diff_empty", &config.text_diff_empty_bg),
    ];

    let fg_color_mappings = [
        ("diff_remove", &config.text_diff_remove_fg),
        ("diff_add", &config.text_diff_add_fg),
        ("diff_empty", &config.text_diff_empty_fg),
    ];

    for buffer in text_buffers {
        // Update background colors
        for (tag_name, color_value) in &bg_color_mappings {
            if let Some(tag) = buffer.tag_table().lookup(tag_name) {
                let rgba = parse_color_with_fallback(color_value, 255, 255, 255, 1.0);
                tag.set_background_rgba(Some(&rgba));
            }
        }

        // Update foreground colors
        for (tag_name, color_value) in &fg_color_mappings {
            if let Some(tag) = buffer.tag_table().lookup(tag_name) {
                let rgba = parse_color_with_fallback(color_value, 0, 0, 0, 1.0);
                tag.set_foreground_rgba(Some(&rgba));
            }
        }
    }
}

/// Get color property from a CSS class using the theme provider.
///
/// # Arguments
///
/// * `class_name` - CSS class name to query
/// * `property` - CSS property name to retrieve
///
/// # Returns
///
/// Returns `Some(gdk::RGBA)` if the color is found, `None` otherwise
pub fn get_css_color(class_name: &str, property: &str) -> Option<gdk::RGBA> {
    parse_css_property(class_name, property)
}

/// Get foreground color from a CSS class.
///
/// # Arguments
///
/// * `class_name` - CSS class name to query
///
/// # Returns
///
/// Returns the foreground color as `gdk::RGBA`, with fallback to gray
pub fn get_color(class_name: &str) -> gdk::RGBA {
    get_css_color(class_name, "color").unwrap_or_else(|| {
        // Fallback to default gray
        gdk::RGBA::new(0.8, 0.8, 0.8, 1.0)
    })
}

/// Get background color from a CSS class.
///
/// # Arguments
///
/// * `class_name` - CSS class name to query
///
/// # Returns
///
/// Returns the background color as `gdk::RGBA`, with fallback to white
pub fn get_background_color(class_name: &str) -> gdk::RGBA {
    get_css_color(class_name, "background-color").unwrap_or_else(|| {
        // Fallback to white
        gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)
    })
}

/// Get background color from a CSS class and return as hex string.
///
/// # Arguments
///
/// * `class_name` - CSS class name to query
///
/// # Returns
///
/// Returns the background color as RGB hex string (e.g., "#ff0000")
pub fn get_background_color_rgb(class_name: &str) -> String {
    let rgba = get_background_color(class_name);
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgba.red() * 255.0) as u8,
        (rgba.green() * 255.0) as u8,
        (rgba.blue() * 255.0) as u8
    )
}

/// Get background color from a CSS class and return as hex string with alpha channel.
///
/// # Arguments
///
/// * `class_name` - CSS class name to query
///
/// # Returns
///
/// Returns the background color as RGBA hex string (e.g., "#ff0000ff")
pub fn get_background_color_rgba(class_name: &str) -> String {
    let rgba = get_background_color(class_name);
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        (rgba.red() * 255.0) as u8,
        (rgba.green() * 255.0) as u8,
        (rgba.blue() * 255.0) as u8,
        (rgba.alpha() * 255.0) as u8
    )
}

/// Get foreground color from a CSS class and return as hex string.
///
/// # Arguments
///
/// * `class_name` - CSS class name to query
///
/// # Returns
///
/// Returns the foreground color as RGB hex string (e.g., "#ff0000")
pub fn get_color_rgb(class_name: &str) -> String {
    let rgba = get_color(class_name);
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgba.red() * 255.0) as u8,
        (rgba.green() * 255.0) as u8,
        (rgba.blue() * 255.0) as u8
    )
}

/// Get foreground color from a CSS class and return as hex string with alpha channel.
///
/// # Arguments
///
/// * `class_name` - CSS class name to query
///
/// # Returns
///
/// Returns the foreground color as RGBA hex string (e.g., "#ff0000ff")
#[allow(dead_code)]
pub fn get_color_rgba(class_name: &str) -> String {
    let rgba = get_color(class_name);
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        (rgba.red() * 255.0) as u8,
        (rgba.green() * 255.0) as u8,
        (rgba.blue() * 255.0) as u8,
        (rgba.alpha() * 255.0) as u8
    )
}

/// Parse any CSS property value as color using centralized parser.
///
/// # Arguments
///
/// * `class_name` - CSS class name to search in
/// * `property` - CSS property name to extract
///
/// # Returns
///
/// Returns `Some(gdk::RGBA)` if the property is found and parsed, `None` otherwise
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

                // Handle both 'background' and 'background-color' for background property
                let property_matches = if property == "background-color" {
                    rule.starts_with("background-color") || rule.starts_with("background")
                } else {
                    rule.starts_with(property)
                };

                if property_matches && let Some(value_part) = rule.split(':').nth(1) {
                    let value = value_part.trim();
                    return Some(parse_color_with_fallback(value, 128, 128, 128, 1.0));
                }
            }
        }
    }

    None
}
