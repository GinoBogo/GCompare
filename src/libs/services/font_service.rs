//! Font service for detecting system fonts.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use std::process::Command;

/// Font information with alias detection
#[derive(Debug, Clone)]
pub struct FontInfo {
    pub name: String,
    pub is_alias: bool,
}

impl FontInfo {
    /// Create a new font information structure.
    ///
    /// # Arguments
    ///
    /// * `name` - The font family name
    /// * `is_alias` - Whether this font is an alias for another font
    pub fn new(name: String, is_alias: bool) -> Self {
        Self { name, is_alias }
    }
}

/// Service for managing font detection and selection.
#[derive(Clone)]
pub struct FontService;

impl FontService {
    /// Create a new font service.
    pub fn new() -> Self {
        Self
    }

    /// Get a list of available monospace fonts on the system.
    ///
    /// Searches for monospace fonts using system font utilities and filters
    /// out style variants to provide clean font family names.
    ///
    /// # Returns
    ///
    /// * `Vec<FontInfo>` - Vector of available monospace font information
    pub fn get_monospace_fonts(&self) -> Vec<FontInfo> {
        let mut fonts = Vec::new();

        // First try to get monospace fonts specifically
        let output = {
            let mut cmd = Command::new("fc-list");
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            cmd.output().ok()
        };

        if let Some(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if !line.is_empty() && line.to_lowercase().contains("mono") {
                    // Parse format: "/path/to/font.ttf: Font Name,Font Name Style:style=Style"
                    if let Some(font_part) = line.split(':').nth(1) {
                        let font_names = font_part.trim();
                        // Split by comma to get individual font names
                        for font_name in font_names.split(',') {
                            let clean_name = font_name.trim();
                            if !clean_name.is_empty()
                                    && !clean_name.to_lowercase().contains("light")
                                    && !clean_name.to_lowercase().contains("bold")
                                    && !clean_name.to_lowercase().contains("italic")
                                    && !clean_name.to_lowercase().contains("thin")
                                    && !clean_name.to_lowercase().contains("black")
                                    && !clean_name.to_lowercase().contains("medium")
                                    && !clean_name.to_lowercase().contains("ret") // Retina
                                    && !clean_name.to_lowercase().contains("med") // Medium
                                    && !clean_name.to_lowercase().contains("sembd") // Semi Bold
                                    && !clean_name.to_lowercase().contains("cond") // Condensed
                                    && !clean_name.to_lowercase().contains("ext")
                            // Extended
                            {
                                if !fonts.iter().any(|f: &FontInfo| f.name == clean_name) {
                                    fonts.push(FontInfo::new(clean_name.to_string(), false));
                                }
                            }
                        }
                    }
                }
            }
        }

        // If we didn't find enough fonts, try a broader search
        let output = if fonts.len() < 5 {
            let mut cmd = Command::new("fc-list");
            cmd.args([":family", ":style"]);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            cmd.output().ok()
        } else {
            None
        };

        if let Some(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    // Parse format like "Font Name:style=Style" or "/path/to/font: Font Name:style=Style"
                    let font_name = if line.contains('/') {
                        // Extract font name from path format
                        line.split(':').nth(1).unwrap_or("").trim()
                    } else {
                        // Extract font name from simple format
                        line.split(':').next().unwrap_or("").trim()
                    };

                    // Check if this might be a monospace font and is not a style variant
                    if (font_name.to_lowercase().contains("mono")
                        || font_name.to_lowercase().contains("code")
                        || font_name.to_lowercase().contains("console")
                        || font_name.to_lowercase().contains("terminal"))
                        && !fonts.iter().any(|f: &FontInfo| f.name == font_name)
                    {
                        fonts.push(FontInfo::new(font_name.to_string(), false));
                    }
                }
            }
        }

        // Add common font aliases that are not installed but map to available fonts
        let common_aliases = vec![
            ("Courier", "Courier"),
            ("Courier New", "Courier New"),
            ("Consolas", "Consolas"),
            ("Menlo", "Menlo"),
        ];

        for (alias, _) in common_aliases {
            if !fonts.iter().any(|f| f.name == alias) {
                fonts.push(FontInfo::new(alias.to_string(), true));
            }
        }

        // Sort fonts alphabetically (aliases will be mixed in)
        fonts.sort_by(|a, b| a.name.cmp(&b.name));

        // If no fonts were detected, fall back to common monospace fonts
        if fonts.is_empty() {
            fonts.extend_from_slice(&[
                FontInfo::new("Monospace".to_string(), true),
                FontInfo::new("Courier New".to_string(), true),
                FontInfo::new("Consolas".to_string(), true),
                FontInfo::new("Menlo".to_string(), true),
                FontInfo::new("DejaVu Sans Mono".to_string(), true),
                FontInfo::new("Liberation Mono".to_string(), true),
                FontInfo::new("Ubuntu Mono".to_string(), true),
                FontInfo::new("Source Code Pro".to_string(), true),
                FontInfo::new("Fira Code".to_string(), true),
                FontInfo::new("Hack".to_string(), true),
                FontInfo::new("JetBrains Mono".to_string(), true),
                FontInfo::new("IBM Plex Mono".to_string(), true),
                FontInfo::new("Noto Sans Mono".to_string(), true),
                FontInfo::new("Hack Nerd Font Mono".to_string(), true),
                FontInfo::new("Adwaita Mono".to_string(), true),
                FontInfo::new("Nimbus Mono PS".to_string(), true),
            ]);
        }

        fonts
    }

    /// Get the best monospace font match for a given font family.
    ///
    /// # Arguments
    ///
    /// * `font_family` - Font family name to find match for
    ///
    /// # Returns
    ///
    /// * `String` - Matched font family name or "Monospace" fallback
    pub fn get_best_monospace_match(&self, font_family: &str) -> String {
        let output = {
            let mut cmd = Command::new("fc-match");
            cmd.args(["-f", "%{family}", font_family]);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            cmd.output().ok()
        };

        if let Some(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let font_family = stdout.trim();
            if !font_family.is_empty() {
                return font_family.to_string();
            }
        }

        // Fallback to "Monospace" if nothing matches
        "Monospace".to_string()
    }
}
