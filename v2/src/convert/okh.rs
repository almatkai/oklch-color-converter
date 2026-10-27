use crate::math::JsMath;
// Native translation of Culori's Björn Ottosson OKHSL/OKHSV implementation.
// MIT copyright (c) 2021 Björn Ottosson. Full notice: LICENSE-OKHSL.
use super::{oklab_to_lrgb, Color, PI};
use crate::color::{max, min, truthy};
fn toe_inv(x: f64) -> f64 {
    (x * x + 0.206 * x) / ((1.0 + 0.206) / (1.0 + 0.03) * (x + 0.03))
}
fn saturation(a: f64, b: f64) -> f64 {
    let (k0, k1, k2, k3, k4, wl, wm, ws) = if -1.88170328 * a - 0.80936493 * b > 1.0 {
        (
            1.19086277,
            1.76576728,
            0.59662641,
            0.75515197,
            0.56771245,
            4.0767416621,
            -3.3077115913,
            0.2309699292,
        )
    } else if 1.81444104 * a - 1.19445276 * b > 1.0 {
        (
            0.73956515,
            -0.45954404,
            0.08285427,
            0.1254107,
            0.14503204,
            -1.2684380046,
            2.6097574011,
            -0.3413193965,
        )
    } else {
        (
            1.35733652,
            -0.00915799,
            -1.1513021,
            -0.50559606,
            0.00692167,
            -0.0041960863,
            -0.7034186147,
            1.707614701,
        )
    };
    let s = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;
    let kl = 0.3963377774 * a + 0.2158037573 * b;
    let km = -0.1055613458 * a - 0.0638541728 * b;
    let ks = -0.0894841775 * a - 1.291485548 * b;
    let ll = 1.0 + s * kl;
    let mm = 1.0 + s * km;
    let ss = 1.0 + s * ks;
    let l = ll * ll * ll;
    let m = mm * mm * mm;
    let sv = ss * ss * ss;
    let ld = 3.0 * kl * ll * ll;
    let md = 3.0 * km * mm * mm;
    let sd = 3.0 * ks * ss * ss;
    let ld2 = 6.0 * kl * kl * ll;
    let md2 = 6.0 * km * km * mm;
    let sd2 = 6.0 * ks * ks * ss;
    let f = wl * l + wm * m + ws * sv;
    let f1 = wl * ld + wm * md + ws * sd;
    let f2 = wl * ld2 + wm * md2 + ws * sd2;
    s - f * f1 / (f1 * f1 - 0.5 * f * f2)
}
fn cusp(a: f64, b: f64) -> [f64; 2] {
    let s = saturation(a, b);
    let [r, g, bl] = oklab_to_lrgb(Color::new("oklab", [1.0, s * a, s * b], None)).v();
    let l = (1.0 / max(max(r, g), bl)).js_cbrt();
    [l, l * s]
}
fn st(c: [f64; 2]) -> [f64; 2] {
    [c[1] / c[0], c[1] / (1.0 - c[0])]
}
fn intersect(a: f64, b: f64, l1: f64, c1: f64, l0: f64, cusp: [f64; 2]) -> f64 {
    if (l1 - l0) * cusp[1] - (cusp[0] - l0) * c1 <= 0.0 {
        return cusp[1] * l0 / (c1 * cusp[0] + cusp[1] * (l0 - l1));
    }
    let mut t = cusp[1] * (l0 - 1.0) / (c1 * (cusp[0] - 1.0) + cusp[1] * (l0 - l1));
    let dl = l1 - l0;
    let dc = c1;
    let kl = 0.3963377774 * a + 0.2158037573 * b;
    let km = -0.1055613458 * a - 0.0638541728 * b;
    let ks = -0.0894841775 * a - 1.291485548 * b;
    let ldt = dl + dc * kl;
    let mdt = dl + dc * km;
    let sdt = dl + dc * ks;
    let light = l0 * (1.0 - t) + t * l1;
    let chroma = t * c1;
    let ll = light + chroma * kl;
    let mm = light + chroma * km;
    let ss = light + chroma * ks;
    let l = ll * ll * ll;
    let m = mm * mm * mm;
    let s = ss * ss * ss;
    let ld = 3.0 * ldt * ll * ll;
    let md = 3.0 * mdt * mm * mm;
    let sd = 3.0 * sdt * ss * ss;
    let ld2 = 6.0 * ldt * ldt * ll;
    let md2 = 6.0 * mdt * mdt * mm;
    let sd2 = 6.0 * sdt * sdt * ss;
    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s - 1.0;
    let r1 = 4.0767416621 * ld - 3.3077115913 * md + 0.2309699292 * sd;
    let r2 = 4.0767416621 * ld2 - 3.3077115913 * md2 + 0.2309699292 * sd2;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s - 1.0;
    let g1 = -1.2684380046 * ld + 2.6097574011 * md - 0.3413193965 * sd;
    let g2 = -1.2684380046 * ld2 + 2.6097574011 * md2 - 0.3413193965 * sd2;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s - 1.0;
    let b1 = -0.0041960863 * ld - 0.7034186147 * md + 1.707614701 * sd;
    let b2 = -0.0041960863 * ld2 - 0.7034186147 * md2 + 1.707614701 * sd2;
    let ur = r1 / (r1 * r1 - 0.5 * r * r2);
    let ug = g1 / (g1 * g1 - 0.5 * g * g2);
    let ub = b1 / (b1 * b1 - 0.5 * b * b2);
    let tr = if ur >= 0.0 { -r * ur } else { 1e6 };
    let tg = if ug >= 0.0 { -g * ug } else { 1e6 };
    let tb = if ub >= 0.0 { -b * ub } else { 1e6 };
    t += min(tr, min(tg, tb));
    t
}
fn cs(l: f64, a: f64, b: f64) -> [f64; 3] {
    let cusp = cusp(a, b);
    let cm = intersect(a, b, l, 1.0, l, cusp);
    let st = st(cusp);
    let sm = 0.11516993
        + 1.0
            / (7.4477897
                + 4.1590124 * b
                + a * (-2.19557347
                    + 1.75198401 * b
                    + a * (-2.13704948 - 10.02301043 * b
                        + a * (-4.24894561 + 5.38770819 * b + 4.69891013 * a))));
    let tm = 0.11239642
        + 1.0
            / (1.6132032 - 0.68124379 * b
                + a * (0.40370612
                    + 0.90148123 * b
                    + a * (-0.27087943
                        + 0.6122399 * b
                        + a * (0.00299215 - 0.45399568 * b - 0.14661872 * a))));
    let k = cm / min(l * st[0], (1.0 - l) * st[1]);
    let ca = l * sm;
    let cb = (1.0 - l) * tm;
    let mid = 0.9
        * k
        * (1.0 / (1.0 / (ca * ca * ca * ca) + 1.0 / (cb * cb * cb * cb)))
            .sqrt()
            .sqrt();
    let ca = l * 0.4;
    let cb = (1.0 - l) * 0.8;
    [(1.0 / (1.0 / (ca * ca) + 1.0 / (cb * cb))).sqrt(), mid, cm]
}
pub(super) fn okhsl_to_oklab(c: Color) -> Color {
    let [h, s, light] = c.v();
    let l = toe_inv(light);
    if !truthy(s) || light == 1.0 {
        return Color::new("oklab", [l, 0.0, 0.0], c.alpha);
    }
    let a = (h / 180.0 * PI).js_cos();
    let b = (h / 180.0 * PI).js_sin();
    let [c0, cm, cmax] = cs(l, a, b);
    let (t, k0, k1, k2) = if s < 0.8 {
        let k1 = 0.8 * c0;
        (1.25 * s, 0.0, k1, 1.0 - k1 / cm)
    } else {
        let k1 = (0.2 * cm * cm * 1.25 * 1.25) / c0;
        (5.0 * (s - 0.8), cm, k1, 1.0 - k1 / (cmax - cm))
    };
    let chroma = k0 + t * k1 / (1.0 - k2 * t);
    Color::new("oklab", [l, chroma * a, chroma * b], c.alpha)
}
pub(super) fn okhsv_to_oklab(c: Color) -> Color {
    let [h, s, v] = c.v();
    let h = if truthy(h) { h } else { 0.0 };
    let a = (h / 180.0 * PI).js_cos();
    let b = (h / 180.0 * PI).js_sin();
    let [sm, t] = st(cusp(a, b));
    let s0 = 0.5;
    let k = 1.0 - s0 / sm;
    let lv = 1.0 - s * s0 / (s0 + t - t * k * s);
    let cv = s * t * s0 / (s0 + t - t * k * s);
    let lvt = toe_inv(lv);
    let cvt = cv * lvt / lv;
    let [r, g, bl] = oklab_to_lrgb(Color::new("oklab", [lvt, a * cvt, b * cvt], None)).v();
    let scale = (1.0 / max(max(max(r, g), bl), 0.0)).js_cbrt();
    let ln = toe_inv(v * lv);
    let chroma = cv * ln / lv;
    Color::new(
        "oklab",
        [ln * scale, chroma * a * scale, chroma * b * scale],
        c.alpha,
    )
}
