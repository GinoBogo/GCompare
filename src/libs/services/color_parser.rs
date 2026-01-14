//! Color parsing utilities for consistent hex and RGBA color handling.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::gdk;

/// Comprehensive color parsing result with alpha channel information.
#[derive(Debug, Clone)]
pub struct ColorParseResult {
    pub rgba: gdk::RGBA,
    pub alpha: f64,
}

impl ColorParseResult {
    /// Create a new color parse result.
    pub fn new(rgba: gdk::RGBA, alpha: f64) -> Self {
        Self { rgba, alpha }
    }
}

/// Parse color string in various formats (hex, rgba, etc).
///
/// Supports:
/// - 6-digit hex: "#RRGGBB" or "RRGGBB"
/// - 8-digit hex: "#RRGGBBAA" or "RRGGBBAA"
/// - 3-digit hex: "#RGB" or "RGB" (expanded to 6-digit)
/// - RGBA format: "rgba(255, 255, 255, 0.8)"
///
/// # Arguments
///
/// * `color_str` - Color string in any supported format
///
/// # Returns
///
/// * `Ok(ColorParseResult)` - Successfully parsed color with RGBA and alpha
/// * `Err(String)` - Error message describing parsing failure
pub fn parse_color_comprehensive(color_str: &str) -> Result<ColorParseResult, String> {
    let trimmed = color_str.trim();

    // Try hex format first
    if let Some(hex_part) = trimmed.strip_prefix('#') {
        parse_hex_color(hex_part)
    } else if trimmed.starts_with("rgba(") {
        parse_rgba_color(trimmed)
    } else if trimmed.contains('#') {
        // Handle cases where # might be in the middle
        if let Some(hex_part) = trimmed.split('#').nth(1) {
            parse_hex_color(hex_part)
        } else {
            parse_hex_color(trimmed) // Try as hex without prefix
        }
    } else {
        // Try as hex without prefix
        parse_hex_color(trimmed)
    }
}

/// Parse hex color string (without # prefix) to RGBA.
///
/// Supports 3-digit and 6-digit hex formats.
///
/// # Arguments
///
/// * `hex` - Hex color string without # prefix
///
/// # Returns
///
/// * `Ok(ColorParseResult)` - Successfully parsed color
/// * `Err(String)` - Error message for invalid hex format
fn parse_hex_color(hex: &str) -> Result<ColorParseResult, String> {
    let cleaned = hex
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>();

    let (r, g, b, a) = match cleaned.len() {
        3 => {
            // 3-digit hex: RGB -> RRGGBB, alpha = 1.0
            let r = expand_hex_digit(cleaned.chars().next().unwrap())?;
            let g = expand_hex_digit(cleaned.chars().nth(1).unwrap())?;
            let b = expand_hex_digit(cleaned.chars().nth(2).unwrap())?;
            (r, g, b, 255)
        }
        6 => {
            // 6-digit hex: RRGGBB, alpha = 1.0
            let r = u8::from_str_radix(&cleaned[0..2], 16)
                .map_err(|_| format!("Invalid red component: {}", &cleaned[0..2]))?;
            let g = u8::from_str_radix(&cleaned[2..4], 16)
                .map_err(|_| format!("Invalid green component: {}", &cleaned[2..4]))?;
            let b = u8::from_str_radix(&cleaned[4..6], 16)
                .map_err(|_| format!("Invalid blue component: {}", &cleaned[4..6]))?;
            (r, g, b, 255)
        }
        8 => {
            // 8-digit hex: RRGGBBAA
            let r = u8::from_str_radix(&cleaned[0..2], 16)
                .map_err(|_| format!("Invalid red component: {}", &cleaned[0..2]))?;
            let g = u8::from_str_radix(&cleaned[2..4], 16)
                .map_err(|_| format!("Invalid green component: {}", &cleaned[2..4]))?;
            let b = u8::from_str_radix(&cleaned[4..6], 16)
                .map_err(|_| format!("Invalid blue component: {}", &cleaned[4..6]))?;
            let a = u8::from_str_radix(&cleaned[6..8], 16)
                .map_err(|_| format!("Invalid alpha component: {}", &cleaned[6..8]))?;
            (r, g, b, a)
        }
        _ => {
            return Err(format!(
                "Invalid hex color length: {}. Expected 3, 6, or 8 hex digits.",
                cleaned.len()
            ));
        }
    };

    let rgba = gdk::RGBA::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    );

    Ok(ColorParseResult::new(rgba, a as f64 / 255.0))
}

/// Expand a single hex digit to two digits (e.g., 'F' -> 'FF').
///
/// # Arguments
///
/// * `digit` - Single hexadecimal character to expand
///
/// # Returns
///
/// * `Ok(u8)` - Expanded byte value
/// * `Err(String)` - Error for invalid hex digit
fn expand_hex_digit(digit: char) -> Result<u8, String> {
    let hex_str = format!("{}{}", digit, digit);
    u8::from_str_radix(&hex_str, 16).map_err(|_| format!("Invalid hex digit: {}", digit))
}

/// Parse rgba() format string to RGBA.
///
/// # Arguments
///
/// * `rgba_str` - RGBA format string (e.g., "rgba(255, 255, 255, 0.8)")
///
/// # Returns
///
/// * `Ok(ColorParseResult)` - Successfully parsed color
/// * `Err(String)` - Error message for invalid RGBA format
fn parse_rgba_color(rgba_str: &str) -> Result<ColorParseResult, String> {
    if !rgba_str.starts_with("rgba(") || !rgba_str.ends_with(')') {
        return Err("Invalid rgba() format".to_string());
    }

    let inner = &rgba_str[5..rgba_str.len() - 1]; // Remove "rgba(" and ")"
    let parts: Vec<&str> = inner.split(',').collect();

    if parts.len() != 4 {
        return Err(format!("rgba() expects 4 components, got {}", parts.len()));
    }

    let r = parts[0]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid red component: {}", parts[0]))?;
    let g = parts[1]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid green component: {}", parts[1]))?;
    let b = parts[2]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid blue component: {}", parts[2]))?;
    let a = parts[3]
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Invalid alpha component: {}", parts[3]))?;

    // Clamp alpha to 0.0-1.0 range
    let a = a.clamp(0.0, 1.0);

    let rgba = gdk::RGBA::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32,
    );

    Ok(ColorParseResult::new(rgba, a))
}

/// Convenience function to parse color and return only RGBA
/// (for backward compatibility).
///
/// This function provides fallback values for common use cases
/// where you just need RGBA and want sensible defaults on parsing failure.
///
/// # Arguments
///
/// * `color_str` - Color string to parse
/// * `fallback_r` - Red component fallback (0-255)
/// * `fallback_g` - Green component fallback (0-255)
/// * `fallback_b` - Blue component fallback (0-255)
/// * `fallback_a` - Alpha component fallback (0.0-1.0)
///
/// # Returns
///
/// * `gdk::RGBA` - Parsed color or fallback
pub fn parse_color_with_fallback(
    color_str: &str,
    fallback_r: u8,
    fallback_g: u8,
    fallback_b: u8,
    fallback_a: f64,
) -> gdk::RGBA {
    parse_color_comprehensive(color_str)
        .map(|result| result.rgba)
        .unwrap_or_else(|_| {
            gdk::RGBA::new(
                fallback_r as f32 / 255.0,
                fallback_g as f32 / 255.0,
                fallback_b as f32 / 255.0,
                fallback_a as f32,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_6_digit() {
        let result = parse_color_comprehensive("#FF0000").unwrap();
        assert_eq!(result.rgba.red(), 1.0);
        assert_eq!(result.rgba.green(), 0.0);
        assert_eq!(result.rgba.blue(), 0.0);
        assert_eq!(result.alpha, 1.0);
    }

    #[test]
    fn test_parse_hex_8_digit() {
        let result = parse_color_comprehensive("#FF000080").unwrap();
        assert_eq!(result.rgba.red(), 1.0);
        assert_eq!(result.rgba.green(), 0.0);
        assert_eq!(result.rgba.blue(), 0.0);
        // Allow for floating point precision differences
        assert!((result.alpha - 0.5).abs() < 0.01); // 0x80 / 255.0 ≈ 0.5
    }

    #[test]
    fn test_parse_hex_3_digit() {
        let result = parse_color_comprehensive("#F00").unwrap();
        assert_eq!(result.rgba.red(), 1.0);
        assert_eq!(result.rgba.green(), 0.0);
        assert_eq!(result.rgba.blue(), 0.0);
        assert_eq!(result.alpha, 1.0);
    }

    #[test]
    fn test_parse_rgba_format() {
        let result = parse_color_comprehensive("rgba(255, 0, 0, 0.5)").unwrap();
        assert_eq!(result.rgba.red(), 1.0);
        assert_eq!(result.rgba.green(), 0.0);
        assert_eq!(result.rgba.blue(), 0.0);
        assert_eq!(result.alpha, 0.5);
    }

    #[test]
    fn test_parse_with_fallback() {
        let rgba = parse_color_with_fallback("invalid", 255, 0, 0, 1.0);
        assert_eq!(rgba.red(), 1.0);
        assert_eq!(rgba.green(), 0.0);
        assert_eq!(rgba.blue(), 0.0);
        assert_eq!(rgba.alpha(), 1.0);
    }
}
