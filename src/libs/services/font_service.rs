//! Font service for detecting system fonts.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use std::process::Command;

/// Service for managing font detection and selection.
#[derive(Clone)]
pub struct FontService;

impl FontService {
    /// Create a new font service.
    pub fn new() -> Self {
        Self
    }

    /// Get a list of available monospace fonts on the system.
    pub fn get_monospace_fonts(&self) -> Vec<String> {
        let mut fonts = Vec::new();
        
        // First try to get monospace fonts specifically
        if let Ok(output) = Command::new("fc-list")
            .output()
        {
            if output.status.success() {
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
                                if !clean_name.is_empty() && 
                                   !clean_name.to_lowercase().contains("light") &&
                                   !clean_name.to_lowercase().contains("bold") &&
                                   !clean_name.to_lowercase().contains("italic") &&
                                   !clean_name.to_lowercase().contains("thin") &&
                                   !clean_name.to_lowercase().contains("black") &&
                                   !clean_name.to_lowercase().contains("medium") &&
                                   !clean_name.to_lowercase().contains("ret") &&  // Retina
                                   !clean_name.to_lowercase().contains("med") &&  // Medium
                                   !clean_name.to_lowercase().contains("sembd") && // Semi Bold
                                   !clean_name.to_lowercase().contains("cond") && // Condensed
                                   !clean_name.to_lowercase().contains("ext") {  // Extended
                                    if !fonts.contains(&clean_name.to_string()) {
                                        fonts.push(clean_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // If we didn't find enough fonts, try a broader search
        if fonts.len() < 5 {
            if let Ok(output) = Command::new("fc-list")
                .args(&[":family", ":style"])
                .output()
            {
                if output.status.success() {
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
                            if (font_name.to_lowercase().contains("mono") ||
                                font_name.to_lowercase().contains("code") ||
                                font_name.to_lowercase().contains("console") ||
                                font_name.to_lowercase().contains("terminal")) &&
                               !fonts.contains(&font_name.to_string()) {
                                fonts.push(font_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Sort fonts alphabetically
        fonts.sort();
        
        // If no fonts were detected, fall back to common monospace fonts
        if fonts.is_empty() {
            fonts.extend_from_slice(&[
                "Monospace".to_string(),
                "Courier New".to_string(),
                "Consolas".to_string(),
                "Menlo".to_string(),
                "DejaVu Sans Mono".to_string(),
                "Liberation Mono".to_string(),
                "Ubuntu Mono".to_string(),
                "Source Code Pro".to_string(),
                "Fira Code".to_string(),
                "Hack".to_string(),
                "JetBrains Mono".to_string(),
                "IBM Plex Mono".to_string(),
                "Noto Sans Mono".to_string(),
                "Hack Nerd Font Mono".to_string(),
                "Adwaita Mono".to_string(),
                "Nimbus Mono PS".to_string(),
            ]);
        }

        fonts
    }

    /// Get the best monospace font match for a given font family.
    pub fn get_best_monospace_match(&self, font_family: &str) -> String {
        if let Ok(output) = Command::new("fc-match")
            .args(&[font_family, ":spacing=100"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Extract font family from fc-match output
                // Format is usually: "Family Name:style=Style:file=/path/to/font"
                let font_family = stdout.split(':').next().unwrap_or("").trim();
                if !font_family.is_empty() {
                    return font_family.to_string();
                }
            }
        }
        
        // Fallback to "Monospace" if nothing matches
        "Monospace".to_string()
    }
}
