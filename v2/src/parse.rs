// Native parser translation of Culori 3.3.0 tokenize/parse and the own Figma P3 rule.
use crate::color::{clamp, max, Color};
use regex::Regex;
use std::sync::OnceLock;
#[path = "named.rs"]
mod named;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Number,
    Percent,
    Hue,
    None,
}
#[derive(Clone, Copy, Debug)]
struct Coord {
    kind: Kind,
    value: f64,
}
fn number() -> &'static Regex {
    static RX: OnceLock<Regex> = OnceLock::new();
    RX.get_or_init(|| Regex::new(r"^[+-]?(?:[0-9]*\.[0-9]+|[0-9]+)(?:[eE][+-]?[0-9]+)?").unwrap())
}
fn ident_char(c: char) -> bool {
    !c.is_ascii() || c.is_ascii_alphanumeric() || c == '_' || c == '-'
}
fn ident_start(s: &str) -> bool {
    let mut c = s.chars();
    match c.next() {
        Some('-') => c
            .next()
            .is_some_and(|v| v == '-' || v == '_' || v.is_ascii_alphabetic() || !v.is_ascii()),
        Some(v) => v == '_' || v.is_ascii_alphabetic() || !v.is_ascii(),
        None => false,
    }
}
fn coord(s: &str) -> Option<(Coord, usize)> {
    if let Some(n) = number().find(s) {
        let mut consumed = n.end();
        let mut value = s[..consumed].parse::<f64>().ok()?;
        let rest = &s[consumed..];
        let kind = if ident_start(rest) {
            let end = rest.find(|c: char| !ident_char(c)).unwrap_or(rest.len());
            let unit = &rest[..end];
            consumed += end;
            value *= match unit {
                "deg" => 1.0,
                "rad" => 180.0 / std::f64::consts::PI,
                "grad" => 9.0 / 10.0,
                "turn" => 360.0,
                _ => return None,
            };
            Kind::Hue
        } else if rest.starts_with('%') {
            consumed += 1;
            Kind::Percent
        } else {
            Kind::Number
        };
        return Some((Coord { kind, value }, consumed));
    }
    if ident_start(s) {
        let end = s.find(|c: char| !ident_char(c)).unwrap_or(s.len());
        if &s[..end] == "none" {
            return Some((
                Coord {
                    kind: Kind::None,
                    value: f64::NAN,
                },
                end,
            ));
        }
    }
    None
}
fn hex(s: &str, mode: &'static str) -> Option<Color> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if ![3, 4, 6, 8].contains(&s.len()) || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(s, 16).ok()?;
    let (v, a) = match s.len() {
        3 | 4 => {
            let n = if s.len() == 4 { value >> 4 } else { value };
            (
                [((n >> 8) & 15) * 17, ((n >> 4) & 15) * 17, (n & 15) * 17],
                if s.len() == 4 {
                    Some((value & 15) * 17)
                } else {
                    None
                },
            )
        }
        _ => {
            let n = if s.len() == 8 { value >> 8 } else { value };
            (
                [(n >> 16) & 255, (n >> 8) & 255, n & 255],
                if s.len() == 8 {
                    Some(value & 255)
                } else {
                    None
                },
            )
        }
    };
    Some(Color::new(
        mode,
        v.map(|v| f64::from(v) / 255.0),
        a.map(|v| f64::from(v) / 255.0),
    ))
}
fn figma(s: &str) -> Color {
    let s = s
        .strip_prefix("Figma P3 ")
        .unwrap_or(s)
        .trim_matches(js_space);
    if s.len() == 9 && s.starts_with('#') {
        if let Some(c) = hex(s, "p3") {
            return c;
        }
    }
    Color::new("p3", [0.0; 3], Some(1.0))
}
fn js_space(c: char) -> bool {
    matches!(c,'\u{0009}'..='\u{000d}'|' '|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}
fn alpha(c: Coord) -> Option<f64> {
    match c.kind {
        Kind::None => None,
        Kind::Percent => Some(c.value / 100.0),
        _ => Some(c.value),
    }
}

/// Own command parse semantics, including Figma P3 prefix and bare 8-digit hex.
/// Library callers should use `convert` for enforced input/work limits.
pub fn parse(input: &str) -> Option<Color> {
    if input.len() > crate::MAX_INPUT_BYTES {
        return None;
    }
    if input.starts_with("Figma P3") {
        return Some(figma(input));
    }
    if input.len() == 9
        && input.starts_with('#')
        && input[1..].bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Some(figma(&format!("Figma P3 {input}")));
    }
    if let Some(c) = hex(input, "rgb") {
        return Some(c);
    }
    if let Some(n) = named::named(&input.to_lowercase()) {
        return Some(Color::new(
            "rgb",
            [
                f64::from((n >> 16) & 255) / 255.0,
                f64::from((n >> 8) & 255) / 255.0,
                f64::from(n & 255) / 255.0,
            ],
            None,
        ));
    }
    if input == "transparent" {
        return Some(Color::new("rgb", [0.0; 3], Some(0.0)));
    }
    if input.contains(',') {
        return legacy(input);
    }
    modern(input)
}
fn legacy(input: &str) -> Option<Color> {
    // Unlike modern syntax, the original legacy regex is anchored without trim.
    let (name, body) = input.split_once('(')?;
    let body = body.strip_suffix(')')?;
    if !matches!(name, "rgb" | "rgba" | "hsl" | "hsla") {
        return None;
    }
    let parts: Vec<_> = body.split(',').collect();
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let mut cs = vec![];
    for part in parts {
        let text = part.trim_matches(js_space);
        let (c, n) = coord(text)?;
        if n != text.len() || c.kind == Kind::None {
            return None;
        }
        cs.push(c);
    }
    let a = if cs.len() == 4 {
        if cs[3].kind == Kind::Hue {
            return None;
        }
        alpha(cs[3])
    } else {
        None
    };
    if name.starts_with("rgb") {
        if cs[0..3].iter().any(|v| v.kind != cs[0].kind)
            || !matches!(cs[0].kind, Kind::Number | Kind::Percent)
        {
            return None;
        }
        let divisor = if cs[0].kind == Kind::Number {
            255.0
        } else {
            100.0
        };
        Some(Color::new(
            "rgb",
            [
                cs[0].value / divisor,
                cs[1].value / divisor,
                cs[2].value / divisor,
            ],
            a,
        ))
    } else {
        if !matches!(cs[0].kind, Kind::Number | Kind::Hue)
            || cs[1].kind != Kind::Percent
            || cs[2].kind != Kind::Percent
        {
            return None;
        }
        Some(Color::new(
            "hsl",
            [
                cs[0].value,
                clamp(cs[1].value / 100.0, 1.0),
                clamp(cs[2].value / 100.0, 1.0),
            ],
            a,
        ))
    }
}
fn modern(input: &str) -> Option<Color> {
    let input = input.trim_matches(js_space);
    let open = input.find('(')?;
    let name = &input[..open];
    if !ident_start(name) || !name.chars().all(ident_char) {
        return None;
    }
    let mut body = &input[open + 1..];
    let mut mode = None;
    if name == "color" {
        body = body.trim_start_matches([' ', '\n', '\t']);
        let end = body.find(|c: char| !ident_char(c)).unwrap_or(body.len());
        let profile = &body[..end];
        mode = Some(match profile {
            "srgb" => "rgb",
            "srgb-linear" => "lrgb",
            "display-p3" => "p3",
            "a98-rgb" => "a98",
            "prophoto-rgb" => "prophoto",
            "rec2020" => "rec2020",
            "xyz" | "xyz-d65" => "xyz65",
            "xyz-d50" => "xyz50",
            "--cubehelix" => "cubehelix",
            "--din99o-lab" => "dlab",
            "--din99o-lch" => "dlch",
            "--hsi" => "hsi",
            "--hsv" => "hsv",
            "--jzazbz" => "jab",
            "--jzczhz" => "jch",
            "--lab-d65" => "lab65",
            "--lch-d65" => "lch65",
            "--lchuv" => "lchuv",
            "--luv" => "luv",
            "--okhsl" => "okhsl",
            "--okhsv" => "okhsv",
            "--xyb" => "xyb",
            "--yiq" => "yiq",
            _ => return None,
        });
        body = &body[end..];
    }
    let mut cs = Vec::new();
    let mut a = None;
    let mut closed = false;
    while !body.is_empty() {
        body = body.trim_start_matches([' ', '\n', '\t']);
        if body.is_empty() {
            break;
        }
        if let Some(rest) = body.strip_prefix(')') {
            if !rest.is_empty() {
                return None;
            }
            closed = true;
            break;
        }
        let is_alpha = body.starts_with('/');
        if is_alpha {
            if a.is_some() || cs.len() != 3 {
                return None;
            }
            body = body[1..].trim_start_matches([' ', '\n', '\t']);
        }
        let (c, n) = coord(body)?;
        body = &body[n..];
        if is_alpha {
            if c.kind == Kind::Hue {
                return None;
            }
            a = Some(c);
        } else {
            if a.is_some() || cs.len() == 3 {
                return None;
            }
            cs.push(c);
        }
    }
    // Culori 3.3.0 also accepts EOF with no closing parenthesis.
    let _ = closed;
    if cs.len() != 3 {
        return None;
    }
    let alpha = a.and_then(alpha);
    let mode = if let Some(mode) = mode {
        mode
    } else {
        match name {
            "rgb" | "rgba" => "rgb",
            "hsl" | "hsla" => "hsl",
            "hwb" => "hwb",
            "lab" => "lab",
            "lch" => "lch",
            "oklab" => "oklab",
            "oklch" => "oklch",
            _ => return None,
        }
    };
    let mut v = [None; 3];
    for i in 0..3 {
        let c = cs[i];
        if c.kind == Kind::None {
            continue;
        }
        if name == "color" {
            if c.kind == Kind::Hue {
                return None;
            }
            v[i] = Some(if c.kind == Kind::Percent {
                c.value / 100.0
            } else {
                c.value
            });
            continue;
        }
        let is_hue =
            matches!(mode, "hsl" | "hwb") && i == 0 || matches!(mode, "lch" | "oklch") && i == 2;
        if is_hue {
            if c.kind == Kind::Percent {
                return None;
            }
            v[i] = Some(c.value);
            continue;
        }
        // LCH/OKLCH chroma unusually accepts hue units in the pinned parser.
        if c.kind == Kind::Hue && !(matches!(mode, "lch" | "oklch") && i == 1) {
            return None;
        }
        let n = match mode {
            "rgb" => c.value / if c.kind == Kind::Number { 255.0 } else { 100.0 },
            "hsl" | "hwb" => {
                if c.kind == Kind::Percent {
                    c.value / 100.0
                } else {
                    c.value
                }
            }
            "oklab" | "oklch" => {
                if i == 0 {
                    if c.kind == Kind::Percent {
                        c.value / 100.0
                    } else {
                        c.value
                    }
                } else if c.kind == Kind::Number {
                    c.value
                } else {
                    c.value * 0.4 / 100.0
                }
            }
            "lab" => {
                if i == 0 || c.kind == Kind::Number {
                    c.value
                } else {
                    c.value * 125.0 / 100.0
                }
            }
            "lch" => {
                if i == 0 || c.kind == Kind::Number {
                    c.value
                } else {
                    c.value * 150.0 / 100.0
                }
            }
            _ => return None,
        };
        v[i] = Some(if matches!(mode, "lch" | "oklch") && i == 1 {
            max(0.0, n)
        } else {
            n
        });
    }
    Some(Color {
        mode,
        coords: v,
        alpha,
    })
}
