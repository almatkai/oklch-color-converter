// Generated from immutable Culori 3.3.0 expressions by scripts/generate-matrices.py.
#![allow(clippy::excessive_precision)] // Preserve the original decimal coefficient spellings.
use super::{gamma, linear, sign, Color};
fn a98_linear(v: f64) -> f64 {
    v.abs().powf(563.0 / 256.0) * sign(v)
}
fn rec_linear(v: f64) -> f64 {
    let a = v.abs();
    if a < 0.018053968510807 * 4.5 {
        v / 4.5
    } else {
        (if sign(v) != 0.0 { sign(v) } else { 1.0 })
            * ((a + 1.09929682680944 - 1.0) / 1.09929682680944).powf(1.0 / 0.45)
    }
}
fn pro_linear(v: f64) -> f64 {
    let a = v.abs();
    if a >= 16.0 / 512.0 {
        sign(v) * a.powf(1.8)
    } else {
        v / 16.0
    }
}
pub(super) fn rgb_to_xyz65(c: Color) -> Color {
    let [r, g, b] = c.v().map(linear);
    Color::new(
        "xyz65",
        [
            0.4123907992659593 * r + 0.357584339383878 * g + 0.1804807884018343 * b,
            0.2126390058715102 * r + 0.715168678767756 * g + 0.0721923153607337 * b,
            0.0193308187155918 * r + 0.119194779794626 * g + 0.9505321522496607 * b,
        ],
        c.alpha,
    )
}
pub(super) fn p3_to_xyz65(c: Color) -> Color {
    let [r, g, b] = c.v().map(linear);
    Color::new(
        "xyz65",
        [
            0.486570948648216 * r + 0.265667693169093 * g + 0.1982172852343625 * b,
            0.2289745640697487 * r + 0.6917385218365062 * g + 0.079286914093745 * b,
            0.0 * r + 0.0451133818589026 * g + 1.043944368900976 * b,
        ],
        c.alpha,
    )
}
pub(super) fn a98_to_xyz65(c: Color) -> Color {
    let [r, g, b] = c.v().map(a98_linear);
    Color::new(
        "xyz65",
        [
            0.5766690429101305 * r + 0.1855582379065463 * g + 0.1882286462349947 * b,
            0.297344975250536 * r + 0.6273635662554661 * g + 0.0752914584939979 * b,
            0.0270313613864123 * r + 0.0706888525358272 * g + 0.9913375368376386 * b,
        ],
        c.alpha,
    )
}
pub(super) fn rec2020_to_xyz65(c: Color) -> Color {
    let [r, g, b] = c.v().map(rec_linear);
    Color::new(
        "xyz65",
        [
            0.6369580483012911 * r + 0.1446169035862083 * g + 0.1688809751641721 * b,
            0.262700212011267 * r + 0.6779980715188708 * g + 0.059301716469862 * b,
            0.0 * r + 0.0280726930490874 * g + 1.0609850577107909 * b,
        ],
        c.alpha,
    )
}
pub(super) fn prophoto_to_xyz50(c: Color) -> Color {
    let [r, g, b] = c.v().map(pro_linear);
    Color::new(
        "xyz50",
        [
            0.7977666449006423 * r + 0.1351812974005331 * g + 0.0313477341283922 * b,
            0.2880748288194013 * r + 0.7118352342418731 * g + 0.0000899369387256 * b,
            0.0 * r + 0.0 * g + 0.8251046025104602 * b,
        ],
        c.alpha,
    )
}
pub(super) fn xyz65_to_rgb(c: Color) -> Color {
    let [x, y, z] = c.v();
    Color::new(
        "rgb",
        [
            x * 3.2409699419045226 - y * 1.5373831775700939 - 0.4986107602930034 * z,
            x * -0.9692436362808796 + y * 1.8759675015077204 + 0.0415550574071756 * z,
            x * 0.0556300796969936 - y * 0.2039769588889765 + 1.0569715142428784 * z,
        ]
        .map(gamma),
        c.alpha,
    )
}
pub(super) fn xyz65_to_p3(c: Color) -> Color {
    let [x, y, z] = c.v();
    Color::new(
        "p3",
        [
            x * 2.4934969119414263 - y * 0.9313836179191242 - 0.402710784450717 * z,
            x * -0.8294889695615749 + y * 1.7626640603183465 + 0.0236246858419436 * z,
            x * 0.0358458302437845 - y * 0.0761723892680418 + 0.9568845240076871 * z,
        ]
        .map(gamma),
        c.alpha,
    )
}
pub(super) fn xyz50_to_rgb(c: Color) -> Color {
    let [x, y, z] = c.v();
    Color::new(
        "rgb",
        [
            x * 3.1341359569958707 - y * 1.6173863321612538 - 0.4906619460083532 * z,
            x * -0.978795502912089 + y * 1.916254567259524 + 0.03344273116131949 * z,
            x * 0.07195537988411677 - y * 0.2289768264158322 + 1.405386058324125 * z,
        ]
        .map(gamma),
        c.alpha,
    )
}
pub(super) fn xyz50_to_xyz65(c: Color) -> Color {
    let [x, y, z] = c.v();
    Color::new(
        "xyz65",
        [
            0.9554734527042182 * x - 0.0230985368742614 * y + 0.0632593086610217 * z,
            -0.0283697069632081 * x + 1.0099954580058226 * y + 0.021041398966943 * z,
            0.0123140016883199 * x - 0.0205076964334779 * y + 1.3303659366080753 * z,
        ],
        c.alpha,
    )
}
