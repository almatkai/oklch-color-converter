use crate::math::JsMath;
// Native translation of Culori 3.3.0. Keep operation order and direct edges.
use crate::color::{hue, sign, truthy, Color};
use std::f64::consts::PI;
mod matrices;
use matrices::*;

pub(crate) fn linear(v: f64) -> f64 {
    let a = v.abs();
    if a <= 0.04045 {
        v / 12.92
    } else {
        (if truthy(sign(v)) { sign(v) } else { 1.0 }) * ((a + 0.055) / 1.055).js_powf(2.4)
    }
}
pub(crate) fn gamma(v: f64) -> f64 {
    let a = v.abs();
    if a > 0.0031308 {
        (if truthy(sign(v)) { sign(v) } else { 1.0 }) * (1.055 * a.js_powf(1.0 / 2.4) - 0.055)
    } else {
        v * 12.92
    }
}
fn polar(c: Color, mode: &'static str) -> Color {
    let [_, a, b] = c.v();
    let chroma = (a * a + b * b).sqrt();
    Color {
        mode,
        coords: [
            c.coords[0],
            Some(chroma),
            if truthy(chroma) {
                Some(hue(b.js_atan2(a) * 180.0 / PI))
            } else {
                None
            },
        ],
        alpha: c.alpha,
    }
}
fn cartesian(c: Color, mode: &'static str) -> Color {
    let [l, ch, h] = c.v();
    Color::new(
        mode,
        [
            l,
            if truthy(ch) {
                ch * (h / 180.0 * PI).js_cos()
            } else {
                0.0
            },
            if truthy(ch) {
                ch * (h / 180.0 * PI).js_sin()
            } else {
                0.0
            },
        ],
        c.alpha,
    )
}
fn oklab_from_rgb(c: Color) -> Color {
    let mut out = lrgb_to_oklab(c.map("lrgb", linear));
    if c.coords[0] == c.coords[2] && c.coords[2] == c.coords[1] {
        out.coords[1] = Some(0.0);
        out.coords[2] = Some(0.0);
    }
    out
}
fn lrgb_to_oklab(c: Color) -> Color {
    let [r, g, b] = c.v();
    let l = (0.41222147079999993 * r + 0.5363325363 * g + 0.0514459929 * b).js_cbrt();
    let m = (0.2119034981999999 * r + 0.6806995450999999 * g + 0.1073969566 * b).js_cbrt();
    let s = (0.08830246189999998 * r + 0.2817188376 * g + 0.6299787005000002 * b).js_cbrt();
    Color::new(
        "oklab",
        [
            0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
        ],
        c.alpha,
    )
}
fn oklab_to_lrgb(c: Color) -> Color {
    let [l, a, b] = c.v();
    let l =
        (l * 0.9999999984505198 + 0.39633779217376786 * a + 0.2158037580607588 * b).js_powf(3.0);
    let m = (l_from(c) * 1.0000000088817609 - 0.10556134232365635 * a - 0.06385417477170591 * b)
        .js_powf(3.0);
    let s = (l_from(c) * 1.0000000546724108 - 0.08948418209496575 * a - 1.2914855378640917 * b)
        .js_powf(3.0);
    Color::new(
        "lrgb",
        [
            4.076741661347994 * l - 3.307711590408193 * m + 0.230969928729428 * s,
            -1.2684380040921763 * l + 2.6097574006633715 * m - 0.3413193963102197 * s,
            -0.004196086541837188 * l - 0.7034186144594493 * m + 1.7076147009309444 * s,
        ],
        c.alpha,
    )
}
fn l_from(c: Color) -> f64 {
    c.v()[0]
}

pub(crate) fn to(c: Color, target: &'static str) -> Color {
    if c.mode == target {
        return c;
    }
    match (c.mode, target) {
        ("oklch", "oklab") => cartesian(c, "oklab"),
        ("oklab", "oklch") => polar(c, "oklch"),
        ("okhsl", "oklab") => okh::okhsl_to_oklab(c),
        ("okhsv", "oklab") => okh::okhsv_to_oklab(c),
        ("lrgb", "oklab") => lrgb_to_oklab(c),
        ("oklab", "lrgb") => oklab_to_lrgb(c),
        (_, "rgb") => rgb(c),
        (_, "xyz65") => match c.mode {
            "rgb" => rgb_to_xyz65(c),
            "p3" => p3_to_xyz65(c),
            "a98" => a98_to_xyz65(c),
            "rec2020" => rec2020_to_xyz65(c),
            "xyz50" => xyz50_to_xyz65(c),
            "lab65" => lab_to_xyz(c, true),
            "jab" => jab_to_xyz(c),
            _ => rgb_to_xyz65(rgb(c)),
        },
        (_, "p3") => xyz65_to_p3(if c.mode == "xyz65" {
            c
        } else {
            rgb_to_xyz65(rgb(c))
        }),
        (_, "oklab") => oklab_from_rgb(rgb(c)),
        (_, "oklch") => polar(oklab_from_rgb(rgb(c)), "oklch"),
        _ => unreachable!("internal target"),
    }
}
fn rgb(c: Color) -> Color {
    match c.mode {
        "rgb" => c,
        "p3" => xyz65_to_rgb(p3_to_xyz65(c)),
        "a98" => xyz65_to_rgb(a98_to_xyz65(c)),
        "rec2020" => xyz65_to_rgb(rec2020_to_xyz65(c)),
        "prophoto" => xyz50_to_rgb(prophoto_to_xyz50(c)),
        "xyz50" => xyz50_to_rgb(c),
        "xyz65" => xyz65_to_rgb(c),
        "lrgb" => c.map("rgb", gamma),
        "oklab" => oklab_to_lrgb(c).map("rgb", gamma),
        "oklch" => rgb(cartesian(c, "oklab")),
        "lab" => xyz50_to_rgb(lab_to_xyz(c, false)),
        "lab65" => xyz65_to_rgb(lab_to_xyz(c, true)),
        "lch" => rgb(cartesian(c, "lab")),
        "lch65" => rgb(cartesian(c, "lab65")),
        "hsl" | "hsv" | "hsi" => hs_to_rgb(c),
        "hwb" => {
            let [h, mut w, mut b] = c.v();
            if w + b > 1.0 {
                let s = w + b;
                w /= s;
                b /= s;
            }
            hs_to_rgb(Color::new(
                "hsv",
                [h, if b == 1.0 { 1.0 } else { 1.0 - w / (1.0 - b) }, 1.0 - b],
                c.alpha,
            ))
        }
        "dlch" => rgb(dlch_to_lab(c)),
        "dlab" => rgb(dlch_to_lab(polar(c, "dlch"))),
        "luv" => xyz50_to_rgb(luv_to_xyz(c)),
        "lchuv" => rgb(cartesian(c, "luv")),
        "jab" => xyz65_to_rgb(jab_to_xyz(c)),
        "jch" => rgb(cartesian(c, "jab")),
        "yiq" => {
            let [y, i, q] = c.v();
            Color::new(
                "rgb",
                [
                    y + 0.95608445 * i + 0.6208885 * q,
                    y - 0.27137664 * i - 0.6486059 * q,
                    y - 1.10561724 * i + 1.70250126 * q,
                ],
                c.alpha,
            )
        }
        "xyb" => {
            let [x, y, b] = c.v();
            let bias = 0.0037930732552754493;
            let bc = bias.js_cbrt();
            let l = (x + y + bc).js_powf(3.0) - bias;
            let m = (y - x + bc).js_powf(3.0) - bias;
            let s = (b + y + bc).js_powf(3.0) - bias;
            Color::new(
                "lrgb",
                [
                    11.031566904639861 * l - 9.866943908131562 * m - 0.16462299650829934 * s,
                    -3.2541473810744237 * l + 4.418770377582723 * m - 0.16462299650829934 * s,
                    -3.6588512867136815 * l + 2.7129230459360922 * m + 1.9459282407775895 * s,
                ],
                c.alpha,
            )
            .map("rgb", gamma)
        }
        "cubehelix" => {
            let [h, s, l] = c.v();
            let h = if c.coords[0].is_none() {
                0.0
            } else {
                h + 120.0
            } * (PI / 180.0);
            let amp = if c.coords[1].is_none() {
                0.0
            } else {
                s * l * (1.0 - l)
            };
            Color::new(
                "rgb",
                [
                    l + amp * (-0.14861 * h.js_cos() + 1.78277 * h.js_sin()),
                    l + amp * (-0.29227 * h.js_cos() - 0.90649 * h.js_sin()),
                    l + amp * (1.97294 * h.js_cos() + 0.0 * h.js_sin()),
                ],
                c.alpha,
            )
        }
        "okhsl" => rgb(super::convert::okh::okhsl_to_oklab(c)),
        "okhsv" => rgb(super::convert::okh::okhsv_to_oklab(c)),
        _ => unreachable!("parser only returns registered modes"),
    }
}
fn hs_to_rgb(c: Color) -> Color {
    let [h, s, n] = c.v();
    let h = hue(h);
    let f = ((h / 60.0) % 2.0 - 1.0).abs();
    let (hi, mid, lo) = match c.mode {
        "hsl" => {
            let hi = n + s * if n < 0.5 { n } else { 1.0 - n };
            (hi, hi - (hi - n) * 2.0 * f, 2.0 * n - hi)
        }
        "hsi" => (
            n * (1.0 + s * (3.0 / (2.0 - f) - 1.0)),
            n * (1.0 + s * (3.0 * (1.0 - f) / (2.0 - f) - 1.0)),
            n * (1.0 - s),
        ),
        _ => (n, n * (1.0 - s * f), n * (1.0 - s)),
    };
    let v = if !h.is_finite() {
        [lo, lo, lo]
    } else {
        match (h / 60.0).floor() as i32 {
            0 => [hi, mid, lo],
            1 => [mid, hi, lo],
            2 => [lo, hi, mid],
            3 => [lo, mid, hi],
            4 => [mid, lo, hi],
            5 => [hi, lo, mid],
            _ => [lo, lo, lo],
        }
    };
    Color::new("rgb", v, c.alpha)
}
const D50: [f64; 3] = [0.3457 / 0.3585, 1.0, (1.0 - 0.3457 - 0.3585) / 0.3585];
const D65: [f64; 3] = [0.3127 / 0.3290, 1.0, (1.0 - 0.3127 - 0.3290) / 0.3290];
fn lab_to_xyz(c: Color, d65: bool) -> Color {
    let [l, a, b] = c.v();
    let fy = (l + 16.0) / 116.0;
    let f = |v: f64| {
        if v.js_powf(3.0) > 216.0 / 24389.0 {
            v.js_powf(3.0)
        } else {
            (116.0 * v - 16.0) / (24389.0 / 27.0)
        }
    };
    let d = if d65 { D65 } else { D50 };
    Color::new(
        if d65 { "xyz65" } else { "xyz50" },
        [
            f(a / 500.0 + fy) * d[0],
            f(fy) * d[1],
            f(fy - b / 200.0) * d[2],
        ],
        c.alpha,
    )
}
fn dlch_to_lab(c: Color) -> Color {
    let [l, ch, h] = c.v();
    let theta = 26.0 / 180.0 * PI;
    let factor = 100.0 / (139.0_f64 / 100.0).ln();
    let light = (l / factor).exp() - 1.0;
    let (a, b) = if c.coords[2].is_none() {
        (0.0, 0.0)
    } else {
        let g = ((0.0435 * ch).exp() - 1.0) / 0.075;
        let e = g * (h / 180.0 * PI - theta).js_cos();
        let f = g * (h / 180.0 * PI - theta).js_sin();
        (
            e * theta.js_cos() - f / 0.83 * theta.js_sin(),
            e * theta.js_sin() + f / 0.83 * theta.js_cos(),
        )
    };
    Color::new("lab65", [light / 0.0039, a, b], c.alpha)
}
fn luv_to_xyz(c: Color) -> Color {
    let [l, u, v] = c.v();
    let un = 4.0 * D50[0] / (D50[0] + 15.0 * D50[1] + 3.0 * D50[2]);
    let vn = 9.0 * D50[1] / (D50[0] + 15.0 * D50[1] + 3.0 * D50[2]);
    let up = u / (13.0 * l) + un;
    let vp = v / (13.0 * l) + vn;
    let y = D50[1]
        * if l <= 8.0 {
            l / (24389.0 / 27.0)
        } else {
            ((l + 16.0) / 116.0).js_powf(3.0)
        };
    Color::new(
        "xyz50",
        [
            y * (9.0 * up) / (4.0 * vp),
            y,
            y * (12.0 - 3.0 * up - 20.0 * vp) / (4.0 * vp),
        ],
        c.alpha,
    )
}
fn jab_to_xyz(c: Color) -> Color {
    let [j, a, b] = c.v();
    let pq = |v: f64| {
        let vp = v.js_powf(1.0 / 134.03437499999998);
        let n = 10000.0
            * ((0.8359375 - vp) / (18.6875 * vp - 18.8515625)).js_powf(1.0 / 0.1593017578125);
        if truthy(n) {
            n
        } else {
            0.0
        }
    };
    let i = (j + 1.6295499532821566e-11) / (0.44 + 0.56 * (j + 1.6295499532821566e-11));
    let l = pq(i + 0.13860504 * a + 0.058047316 * b);
    let m = pq(i - 0.13860504 * a - 0.058047316 * b);
    let s = pq(i - 0.096019242 * a - 0.8118919 * b);
    Color::new(
        "xyz65",
        [
            (1.661373024652174 * l - 0.914523081304348 * m + 0.23136208173913045 * s) / 203.0,
            (-0.3250758611844533 * l + 1.571847026732543 * m - 0.21825383453227928 * s) / 203.0,
            (-0.090982811 * l - 0.31272829 * m + 1.5227666 * s) / 203.0,
        ],
        c.alpha,
    )
}
mod okh;
