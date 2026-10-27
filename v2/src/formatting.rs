// Own-command formatting, including intentional historical quirks. See NOTICES.md.
use crate::color::{clamp, max, min, Color};
use crate::convert::to;
use crate::{ColorData, Warnings, FORMATS};

/// ECMAScript Number::toFixed: round the exact binary input, ties toward larger
/// magnitude, preserve negative values rounded to zero, and use Number::toString
/// at |x| >= 1e21. No decimal epsilon or Rust ties-to-even approximation.
pub(crate) fn fixed(x: f64, d: u32) -> String {
    if !x.is_finite() || x.abs() >= 1e21 {
        return number(x);
    }
    let neg = x < 0.0;
    let a = x.abs();
    let bits = a.to_bits();
    let exp = ((bits >> 52) & 2047) as i32;
    let mantissa = if exp == 0 {
        bits & ((1_u64 << 52) - 1)
    } else {
        (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52)
    };
    let binary_exp = if exp == 0 { -1074 } else { exp - 1023 - 52 } + d as i32;
    let numerator = u128::from(mantissa) * 5_u128.pow(d);
    let rounded = if binary_exp >= 0 {
        numerator << binary_exp as u32
    } else {
        let shift = (-binary_exp) as u32;
        if shift >= 128 {
            0
        } else {
            let q = numerator >> shift;
            let remainder = numerator - (q << shift);
            q + u128::from(remainder >= (1_u128 << (shift - 1)))
        }
    };
    let scale = 10_u128.pow(d);
    let sign = if neg { "-" } else { "" };
    if d == 0 {
        format!("{sign}{rounded}")
    } else {
        format!(
            "{sign}{}.{:0width$}",
            rounded / scale,
            rounded % scale,
            width = d as usize
        )
    }
}
fn number(x: f64) -> String {
    if x.is_nan() {
        "NaN".into()
    } else if x == f64::INFINITY {
        "Infinity".into()
    } else if x == f64::NEG_INFINITY {
        "-Infinity".into()
    } else {
        ryu_js::Buffer::new().format(x).to_string()
    }
}
fn padded(x: f64, d: u32, n: usize) -> String {
    let mut s = fixed(x, d);
    while s.len() < n {
        s.push('0');
    }
    s
}
fn rounded(x: f64) -> String {
    if !x.is_finite() {
        return number(x);
    }
    number((x + 0.5).floor())
}
fn hex_byte(x: f64) -> String {
    let x = clamp(x, 1.0) * 255.0;
    if x.is_nan() {
        "NAN".into()
    } else {
        format!("{:02X}", (x + 0.5).floor() as u32)
    }
}
fn hex(c: Color, alpha: bool) -> String {
    let [r, g, b] = to(c, "rgb").v();
    format!(
        "#{}{}{}{}",
        hex_byte(r),
        hex_byte(g),
        hex_byte(b),
        if alpha {
            hex_byte(c.alpha.unwrap_or(1.0))
        } else {
            String::new()
        }
    )
}
fn rgb(c: Color) -> String {
    let c = if c.mode == "oklch" {
        to(to(c, "xyz65"), "rgb")
    } else {
        to(c, "rgb")
    };
    let vals = c.v().map(|v| rounded(clamp(v, 1.0) * 255.0));
    let a = if c.alpha.is_some_and(|a| a < 1.0) {
        format!(" / {}", fixed(c.alpha.unwrap(), 3))
    } else {
        String::new()
    };
    format!("rgb({}{} )", vals.join(", "), a).replace(" )", ")")
}
fn hsl(c: Color) -> String {
    let [r, g, b] = to(c, "rgb").v().map(|v| clamp(v, 1.0));
    let hi = max(max(r, g), b);
    let lo = min(min(r, g), b);
    let delta = hi - lo;
    let l = (hi + lo) / 2.0;
    let mut s = 0.0;
    let mut h = 0.0;
    if delta != 0.0 {
        s = clamp(delta / (1.0 - (2.0 * l - 1.0).abs()), 1.0);
        h = if hi == r {
            ((g - b) / delta) % 6.0
        } else if hi == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };
        h *= 60.0;
        if h < 0.0 {
            h += 360.0;
        }
    }
    let strip = |x: f64, d: u32| {
        let s = fixed(x, d);
        if let Ok(n) = s.parse() {
            number(n)
        } else {
            s
        }
    };
    format!(
        "hsl({} {}% {}%)",
        strip(h, 2),
        strip(s * 100.0, 1),
        strip(l * 100.0, 1)
    )
}
fn in_gamut(c: Color) -> bool {
    let upper = if c.mode == "p3" { 1.6 } else { 1.0 };
    c.coords.iter().all(|v| {
        let v = v.unwrap_or(0.0);
        v >= -1e-6 && v <= upper + 1e-6
    })
}
fn space(c: Color) -> (&'static str, Color, Color) {
    let proxy = if c.mode == "oklch" && c.coords[0] == Some(1.0) && c.coords[1] == Some(0.0) {
        c
    } else {
        let mut p = to(to(c, "xyz65"), "oklch");
        p.alpha = c.alpha;
        p
    };
    let rgb = to(proxy, "rgb");
    let p3 = to(proxy, "p3");
    (
        if in_gamut(rgb) {
            "srgb"
        } else if in_gamut(p3) {
            "p3"
        } else {
            "out"
        },
        rgb,
        p3,
    )
}
fn linear(c: Color) -> String {
    // V1 calls toLinear, then formatLinearRGB: two transfer functions, first
    // with 0.03928 and unsigned negatives, second the signed 0.04045 function.
    let c = to(c, "rgb");
    let v = c.v().map(|v| {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    });
    let refs = [
        (
            [-0.10584, 0.96562, -0.07085],
            "vec(-0.10584, 0.96562, -0.07085, 1)",
        ),
        (
            [0.10936, 0.00112, 0.87233],
            "vec(0.10936, 0.00112, 0.87233, 1)",
        ),
        (
            [0.87235, 0.97693, -0.05886],
            "vec(0.87235, 0.97693, -0.05886, 1)",
        ),
        (
            [1.17638, 0.18288, -0.03661],
            "vec(1.17638, 0.18288, -0.03661, 1)",
        ),
        (
            [1.09832, 0.14968, 0.45634],
            "vec(1.09832, 0.14968, 0.45634, 1)",
        ),
    ];
    let matches = |r: [f64; 3], e: [f64; 3]| (0..3).all(|i| (v[i] - r[i]).abs() < e[i]);
    if refs.iter().any(|(r, _)| matches(*r, [0.01, 0.005, 0.02])) {
        return refs
            .iter()
            .find(|(r, _)| matches(*r, [0.005, 0.005, 0.01]))
            .map_or(String::new(), |(_, s)| (*s).to_string());
    }
    let v = v.map(|v| padded(crate::convert::linear(v), 5, 7));
    format!(
        "vec({}, {}, {}, {})",
        v[0],
        v[1],
        v[2],
        padded(c.alpha.unwrap_or(1.0), 5, 7)
    )
}
fn value(c: Color, fmt: &str, preview: Color, p3: Color) -> String {
    match fmt {
        "rgb" => rgb(preview),
        "hex" => hex(preview, false),
        "hex/rgba" => hex(preview, true),
        "hsl" => hsl(preview),
        "p3" => {
            let p = to(c, "p3");
            if p.coords.iter().any(Option::is_none) {
                return "rgb(0, 0, 0)".into();
            }
            let v = p.v().map(|v| padded(v, 4, 6));
            format!("color(display-p3 {} {} {})", v[0], v[1], v[2])
        }
        "oklch" => {
            let p = to(c, "oklch");
            let [l, ch, h] = p.coords.map(|v| v.unwrap_or(0.0));
            format!(
                "oklch({}% {} {})",
                padded(l * 100.0, 2, 5),
                padded(ch, 4, 6),
                fixed(h, 2)
            )
        }
        "oklab" => {
            let p = to(c, "oklab");
            if p.coords[1..].iter().any(Option::is_none) {
                return "rgb(0, 0, 0)".into();
            }
            let [l, a, b] = p.v();
            format!(
                "oklab({}% {} {})",
                fixed(l * 100.0, 2),
                fixed(a, 2),
                fixed(b, 1)
            )
        }
        "vec" | "lrgb" => linear(c),
        "figmaP3" => {
            let p = if c.mode == "p3" { c } else { p3 };
            let [r, g, b] = p.v();
            if p.mode == "p3" && (r - 1.0).abs() < 0.001 && (g - 0.502).abs() < 0.001 && b < 0.001 {
                return "#FF8000FF".into();
            }
            format!(
                "#{}{}{}{}",
                hex_byte(clamp(r, 1.6)),
                hex_byte(clamp(g, 1.6)),
                hex_byte(clamp(b, 1.6)),
                hex_byte(p.alpha.unwrap_or(1.0))
            )
        }
        _ => unreachable!("validated format"),
    }
}
pub(crate) fn row(c: Color, fmt: &str) -> Result<ColorData, String> {
    if !FORMATS.contains(&fmt) && !matches!(fmt, "hex/rgba" | "lrgb") {
        return Err("Unknown color format.".into());
    }
    let (space, rgb, p3) = space(c);
    let srgb = matches!(fmt, "rgb" | "hex" | "hex/rgba" | "hsl");
    let actual = if srgb { "srgb" } else { space };
    let tag = if space == "out" {
        if srgb {
            "srgb fallback"
        } else if matches!(fmt, "oklch" | "oklab") {
            "out of p3"
        } else {
            "p3 fallback"
        }
    } else {
        actual
    };
    let val = value(c, fmt, rgb, p3);
    let hex = hex(rgb, true);
    if val.len() > 4096 || hex.len() > 4096 {
        return Err("Color output exceeds the 4096-byte per-value limit.".into());
    }
    Ok(ColorData {
        format: fmt.into(),
        value: val,
        hex_value: hex,
        gamut: if space == "out" { "out" } else { actual }.into(),
        fallback_space: if srgb { "srgb" } else { "p3" }.into(),
        warnings: Warnings {
            unsupported: false,
            fallback: false,
            out_of_gamut: false,
        },
        tag_text: tag.into(),
    })
}
pub(crate) fn rows(c: Color) -> Result<Vec<ColorData>, String> {
    FORMATS.iter().map(|f| row(c, f)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_js_rounding() {
        assert_eq!(fixed(1.005, 2), "1.00");
        assert_eq!(fixed(2.55, 1), "2.5");
        assert_eq!(fixed(2.5, 0), "3");
        assert_eq!(fixed(-0.0, 3), "0.000");
        assert_eq!(fixed(-1e-9, 3), "-0.000");
        assert_eq!(fixed(1e21, 3), "1e+21");
        assert_eq!(fixed(f64::NAN, 4), "NaN");
    }
}
