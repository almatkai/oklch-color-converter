//! Native translation of the fetched command and Culori 3.3.0 conversion chains.
//! Original command by strongeron; numeric formula provenance is in NOTICES.md.
mod color;
mod convert;
mod formatting;
mod math;
mod parse;

pub use color::Color;
pub use parse::parse;
use serde::{Deserialize, Serialize};

pub const MAX_INPUT_BYTES: usize = 65_536;
pub const FORMATS: [&str; 8] = [
    "figmaP3", "oklch", "p3", "oklab", "vec", "hex", "rgb", "hsl",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorData {
    pub format: String,
    pub value: String,
    pub hex_value: String,
    pub gamut: String,
    pub fallback_space: String,
    pub warnings: Warnings,
    pub tag_text: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Warnings {
    pub unsupported: bool,
    pub fallback: bool,
    pub out_of_gamut: bool,
}

/// Empty and invalid input produce no rows, as in the original command.
/// Resource violations are explicit errors, never partial results.
pub fn convert(input: &str) -> Result<Vec<ColorData>, String> {
    if input.len() > MAX_INPUT_BYTES {
        return Err("Color input exceeds the 65536-byte limit.".into());
    }
    if input.is_empty() {
        return Ok(vec![]);
    }
    let Some(color) = parse(input) else {
        return Ok(vec![]);
    };
    if color
        .coords
        .iter()
        .flatten()
        .chain(color.alpha.iter())
        .any(|v| !v.is_finite())
    {
        return Err("Color input contains a non-finite number.".into());
    }
    formatting::rows(color)
}

/// Includes the two non-rendered helper formats for reproducible source parity checks.
pub fn format_color(color: Color, format: &str) -> Result<ColorData, String> {
    const MODES: [&str; 29] = [
        "rgb",
        "lrgb",
        "p3",
        "a98",
        "prophoto",
        "rec2020",
        "xyz50",
        "xyz65",
        "oklab",
        "oklch",
        "lab",
        "lab65",
        "lch",
        "lch65",
        "hsl",
        "hsv",
        "hsi",
        "hwb",
        "dlch",
        "dlab",
        "luv",
        "lchuv",
        "jab",
        "jch",
        "yiq",
        "xyb",
        "cubehelix",
        "okhsl",
        "okhsv",
    ];
    if !MODES.contains(&color.mode) {
        return Err("Unknown color mode.".into());
    }
    if color
        .coords
        .iter()
        .flatten()
        .chain(color.alpha.iter())
        .any(|v| !v.is_finite())
    {
        return Err("Color input contains a non-finite number.".into());
    }
    formatting::row(color, format)
}
