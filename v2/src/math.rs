// Native translation of the cbrt algorithm in V8 14.6.202 src/base/ieee754.cc.
// Copyright 1993 Sun Microsystems, Inc. and the V8 project authors.
// See LICENSE-V8 and NOTICES.md. Changed: Rust bit operations and naming.
pub(crate) trait JsMath {
    fn js_cbrt(self) -> f64;
    fn js_sin(self) -> f64;
    fn js_cos(self) -> f64;
    fn js_atan2(self, x: f64) -> f64;
    fn js_powf(self, p: f64) -> f64;
}
impl JsMath for f64 {
    fn js_sin(self) -> f64 {
        libm::sin(self)
    }
    fn js_cos(self) -> f64 {
        libm::cos(self)
    }
    fn js_atan2(self, x: f64) -> f64 {
        libm::atan2(self, x)
    }
    fn js_cbrt(self) -> f64 {
        cbrt(self)
    }
    fn js_powf(self, p: f64) -> f64 {
        libm::pow(self, p)
    }
}
fn cbrt(x: f64) -> f64 {
    let bits = x.to_bits();
    let high = (bits >> 32) as u32;
    let low = bits as u32;
    let sign = high & 0x80000000;
    let hx = high ^ sign;
    if hx >= 0x7ff00000 {
        return x + x;
    }
    let mut t = if hx < 0x00100000 {
        if hx | low == 0 {
            return x;
        }
        let t = f64::from_bits(0x43500000_u64 << 32) * x;
        let high = (t.to_bits() >> 32) as u32;
        f64::from_bits(u64::from(sign | ((high & 0x7fffffff) / 3 + 696219795)) << 32)
    } else {
        f64::from_bits(u64::from(sign | (hx / 3 + 715094163)) << 32)
    };
    let r = (t * t) * (t / x);
    t *= (1.87595182427177 + r * (-1.8849797954337717 + r * 1.6214297201053545))
        + ((r * r) * r) * (-0.758397934778766 + r * 0.14599619288661245);
    t = f64::from_bits((t.to_bits() + 0x80000000) & 0xffffffffc0000000);
    let s = t * t;
    let r = x / s;
    let w = t + t;
    let r = (r - t) / (w + r);
    t + t * r
}
