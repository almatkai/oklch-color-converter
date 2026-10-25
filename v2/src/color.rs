/// Coordinates follow the channel order of the locked Culori mode definition.
/// `None` is an absent CSS `none` coordinate, distinct from a computed NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub mode: &'static str,
    pub coords: [Option<f64>; 3],
    pub alpha: Option<f64>,
}
impl Color {
    pub(crate) fn new(mode: &'static str, v: [f64; 3], alpha: Option<f64>) -> Self {
        Self {
            mode,
            coords: v.map(Some),
            alpha,
        }
    }
    pub(crate) fn v(self) -> [f64; 3] {
        self.coords.map(|v| v.unwrap_or(f64::NAN))
    }
    pub(crate) fn map(self, mode: &'static str, f: impl Fn(f64) -> f64) -> Self {
        Self::new(mode, self.v().map(f), self.alpha)
    }
}
pub(crate) fn truthy(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}
pub(crate) fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}
pub(crate) fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}
pub(crate) fn clamp(v: f64, hi: f64) -> f64 {
    max(0.0, min(hi, v))
}
pub(crate) fn sign(v: f64) -> f64 {
    if v == 0.0 {
        v
    } else {
        v.signum()
    }
}
pub(crate) fn hue(v: f64) -> f64 {
    let h = v % 360.0;
    if h < 0.0 {
        h + 360.0
    } else {
        h
    }
}
