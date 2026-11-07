//! RF calculations use SI units and the exp(j*w*t) convention.
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
pub type C = Complex64;
pub const LIGHT: f64 = 299_792_458.0;
pub fn c(re: f64, im: f64) -> C {
    C::new(re, im)
}
pub fn gamma(z: C, z0: f64) -> C {
    if !z.re.is_finite() || !z.im.is_finite() {
        return c(1., 0.);
    }
    (z - z0) / (z + z0)
}
pub fn impedance(g: C, z0: f64) -> C {
    if (c(1., 0.) - g).norm() < 1e-14 {
        return c(1e30, 0.);
    }
    z0 * (c(1., 0.) + g) / (c(1., 0.) - g)
}
pub fn inv(z: C) -> C {
    if z.norm() < 1e-28 {
        c(1e30, 0.)
    } else {
        c(1., 0.) / z
    }
}
pub fn parallel(a: C, b: C) -> C {
    inv(inv(a) + inv(b))
}
pub fn db(a: f64) -> f64 {
    20. * a.max(1e-30).log10()
}
pub fn vswr(g: C) -> f64 {
    if g.norm() >= 1. {
        f64::INFINITY
    } else {
        (1. + g.norm()) / (1. - g.norm())
    }
}
pub fn quality(z: C) -> f64 {
    if z.re.abs() < 1e-25 {
        f64::INFINITY
    } else {
        (z.im / z.re).abs()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kind {
    SeriesR,
    ShuntR,
    SeriesL,
    ShuntL,
    SeriesC,
    ShuntC,
    Line,
    OpenStub,
    ShortStub,
    Transformer,
    SeriesParallelRlc,
    ShuntSeriesRlc,
}
impl Kind {
    pub const ALL: [Kind; 12] = [
        Self::SeriesR,
        Self::ShuntR,
        Self::SeriesL,
        Self::ShuntL,
        Self::SeriesC,
        Self::ShuntC,
        Self::Line,
        Self::OpenStub,
        Self::ShortStub,
        Self::Transformer,
        Self::SeriesParallelRlc,
        Self::ShuntSeriesRlc,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::SeriesR => "Series R",
            Self::ShuntR => "Shunt R",
            Self::SeriesL => "Series L",
            Self::ShuntL => "Shunt L",
            Self::SeriesC => "Series C",
            Self::ShuntC => "Shunt C",
            Self::Line => "Transmission line",
            Self::OpenStub => "Open stub",
            Self::ShortStub => "Shorted stub",
            Self::Transformer => "Transformer",
            Self::SeriesParallelRlc => "Series (R || L || C)",
            Self::ShuntSeriesRlc => "Shunt (R + L + C)",
        }
    }
    pub fn shunt(self) -> bool {
        matches!(
            self,
            Self::ShuntR
                | Self::ShuntL
                | Self::ShuntC
                | Self::OpenStub
                | Self::ShortStub
                | Self::ShuntSeriesRlc
        )
    }
    pub fn unit(self) -> &'static str {
        match self {
            Self::SeriesR | Self::ShuntR => "ohm",
            Self::SeriesL | Self::ShuntL => "nH",
            Self::SeriesC | Self::ShuntC => "pF",
            Self::Transformer => "turns ratio",
            Self::Line | Self::OpenStub | Self::ShortStub => "wavelengths at reference f",
            _ => "ohm",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Element {
    pub kind: Kind,
    pub value: f64,
    pub l_nh: f64,
    pub c_pf: f64,
    pub z0: f64,
    pub er: f64,
    pub loss: f64,
    pub reference_hz: f64,
    pub enabled: bool,
}
impl Element {
    pub fn new(kind: Kind, f: f64) -> Self {
        Self {
            kind,
            value: match kind {
                Kind::SeriesC | Kind::ShuntC => 10.,
                Kind::SeriesL | Kind::ShuntL => 10.,
                Kind::Transformer => 1.,
                Kind::Line | Kind::OpenStub | Kind::ShortStub => 0.125,
                _ => 50.,
            },
            l_nh: 10.,
            c_pf: 10.,
            z0: 50.,
            er: 1.,
            loss: 0.,
            reference_hz: f,
            enabled: true,
        }
    }
    pub fn physical_m(&self) -> f64 {
        self.value * LIGHT / (self.reference_hz * self.er.sqrt())
    }
    pub fn apply(&self, z: C, f: f64) -> C {
        self.apply_fraction(z, f, 1.)
    }
    /// Interpolates a physically meaningful insertion path, with identity at t=0.
    pub fn apply_fraction(&self, z: C, f: f64, t: f64) -> C {
        if !self.enabled || t <= 0. {
            return z;
        }
        let w = TAU * f;
        let v = self.value.max(1e-24);
        match self.kind {
            Kind::SeriesR => z + c(v * t, 0.),
            Kind::ShuntR => inv(inv(z) + c(t / v, 0.)),
            Kind::SeriesL => z + c(0., w * v * 1e-9 * t),
            Kind::ShuntL => inv(inv(z) + c(0., -t / (w * v * 1e-9))),
            Kind::SeriesC => z + c(0., -t / (w * v * 1e-12)),
            Kind::ShuntC => inv(inv(z) + c(0., w * v * 1e-12 * t)),
            Kind::Transformer => z * (1. + (v * v - 1.) * t),
            Kind::Line => {
                let a =
                    self.loss * (f / self.reference_hz).sqrt() * self.physical_m() / 8.685889638;
                let gl = c(
                    (a * t).min(300.),
                    TAU * self.value * f / self.reference_hz * t,
                );
                let ch = gl.cosh();
                let sh = gl.sinh();
                self.z0 * (z * ch + self.z0 * sh) / (self.z0 * ch + z * sh)
            }
            Kind::OpenStub => {
                let theta = TAU * self.value * f / self.reference_hz;
                inv(inv(z) + c(0., theta.tan() / self.z0 * t))
            }
            Kind::ShortStub => {
                let theta = TAU * self.value * f / self.reference_hz;
                inv(inv(z) + c(0., -t / (self.z0 * theta.tan())))
            }
            Kind::SeriesParallelRlc => {
                z + inv(c(
                    1. / v,
                    w * self.c_pf * 1e-12 - 1. / (w * self.l_nh * 1e-9),
                )) * t
            }
            Kind::ShuntSeriesRlc => {
                inv(inv(z) + inv(c(v, w * self.l_nh * 1e-9 - 1. / (w * self.c_pf * 1e-12))) * t)
            }
        }
    }
}
pub fn transform(z: C, f: f64, e: &[Element]) -> C {
    e.iter().fold(z, |z, e| e.apply(z, f))
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SPoint {
    pub hz: f64,
    /// Touchstone order: 11,21,12,22.
    pub s: [C; 4],
}
#[derive(Debug, Clone, Copy)]
pub struct Stability {
    pub k: f64,
    pub mu: f64,
    pub mu_prime: f64,
    pub gain_db: f64,
    pub stable: bool,
    pub delta: C,
}
impl SPoint {
    pub fn stability(&self) -> Stability {
        let [a, b, d, e] = self.s;
        let delta = a * e - b * d;
        let p = (b * d).norm();
        let k = if p > 1e-30 {
            (1. - a.norm_sqr() - e.norm_sqr() + delta.norm_sqr()) / (2. * p)
        } else {
            f64::INFINITY
        };
        let mu = (1. - a.norm_sqr()) / ((e - delta * a.conj()).norm() + p);
        let mu_prime = (1. - e.norm_sqr()) / ((a - delta * e.conj()).norm() + p);
        let stable = k > 1. && delta.norm() < 1.;
        let gain = if d.norm() < 1e-25 {
            b.norm_sqr() / ((1. - a.norm_sqr()) * (1. - e.norm_sqr()))
        } else if stable {
            (b / d).norm() / (k + (k * k - 1.).sqrt())
        } else {
            (b / d).norm()
        };
        Stability {
            k,
            mu,
            mu_prime,
            gain_db: 10. * gain.max(1e-30).log10(),
            stable,
            delta,
        }
    }
    pub fn stability_circle(&self, source: bool) -> Option<(C, f64)> {
        if self
            .s
            .iter()
            .any(|s| !s.re.is_finite() || !s.im.is_finite())
        {
            return None;
        }
        let [a, b, d, e] = self.s;
        let det = a * e - b * d;
        let s = if source { a } else { e };
        let other = if source { e } else { a };
        let den = s.norm_sqr() - det.norm_sqr();
        if den.abs() < 1e-20 {
            return None;
        }
        Some((
            (s - det * other.conj()).conj() / den,
            (b * d).norm() / den.abs(),
        ))
    }
    pub fn gain_circle(&self, source: bool, gain_db: f64, unilateral: bool) -> Option<(C, f64)> {
        if !gain_db.is_finite()
            || self
                .s
                .iter()
                .any(|s| !s.re.is_finite() || !s.im.is_finite())
        {
            return None;
        }
        let [a, b, d, e] = self.s;
        let gain = 10f64.powf(gain_db / 10.);
        let s = if source { a } else { e };
        if unilateral {
            if s.norm() >= 1. {
                return None;
            }
            let g = gain * (1. - s.norm_sqr());
            let den = 1. - (1. - g) * s.norm_sqr();
            let r = (1. - g).sqrt() * (1. - s.norm_sqr()) / den;
            return r.is_finite().then_some((g * s.conj() / den, r.abs()));
        }
        let stability = self.stability();
        if stability.stable && gain_db > stability.gain_db + 1e-8 {
            return None;
        }
        let det = a * e - b * d;
        let g = gain / b.norm_sqr();
        let other = if source { e } else { a };
        let den = 1. + g * (s.norm_sqr() - det.norm_sqr());
        let rad = 1. - 2. * self.stability().k * g * (b * d).norm() + g * g * (b * d).norm_sqr();
        if rad < 0. || den.abs() < 1e-25 {
            return None;
        }
        Some((
            g * (s - det * other.conj()).conj() / den,
            rad.sqrt() / den.abs(),
        ))
    }
    pub fn to_parameters(&self, kind: &str, z0: f64, normalized: bool) -> Result<[C; 4], String> {
        if kind == "S" {
            return Ok(self.s);
        }
        let [a, b, d, e] = self.s;
        if kind == "A" {
            if b.norm() < 1e-25 {
                return Err("ABCD conversion requires nonzero S21".into());
            }
            let v = [
                ((1. + a) * (1. - e) + b * d) / (2. * b),
                ((1. - a) * (1. - e) - b * d) / (2. * b * z0),
                z0 * ((1. + a) * (1. + e) - b * d) / (2. * b),
                ((1. - a) * (1. + e) + b * d) / (2. * b),
            ];
            return Ok(if normalized {
                [v[0], v[1] * z0, v[2] / z0, v[3]]
            } else {
                v
            });
        }
        if kind == "H" {
            let den = (1. - a) * (1. + e) + b * d;
            if den.norm() < 1e-25 {
                return Err("Singular S to H conversion".into());
            }
            let v = [
                z0 * ((1. + a) * (1. + e) - b * d) / den,
                -2. * b / den,
                2. * d / den,
                ((1. - a) * (1. - e) - b * d) / (den * z0),
            ];
            return Ok(if normalized {
                [v[0] / z0, v[1], v[2], v[3] * z0]
            } else {
                v
            });
        }
        let den = (1. - a) * (1. - e) - b * d;
        if den.norm() < 1e-24 {
            return Err("Singular S to Z conversion".into());
        }
        let z = [
            z0 * ((1. + a) * (1. - e) + b * d) / den,
            2. * z0 * b / den,
            2. * z0 * d / den,
            z0 * ((1. + e) * (1. - a) + b * d) / den,
        ];
        let det = z[0] * z[3] - z[1] * z[2];
        let out = match kind {
            "Z" => z,
            "Y" => [z[3] / det, -z[1] / det, -z[2] / det, z[0] / det],
            "H" => [det / z[3], -z[1] / z[3], z[2] / z[3], inv(z[3])],
            "A" => [z[0] / z[1], inv(z[1]), det / z[1], z[3] / z[1]],
            _ => return Err("Unknown parameter kind".into()),
        };
        if !out.iter().all(|x| x.re.is_finite() && x.im.is_finite()) {
            return Err("Singular parameter conversion".into());
        }
        Ok(if normalized {
            match kind {
                "Z" => out.map(|z| z / z0),
                "Y" => out.map(|z| z * z0),
                "H" => [out[0] / z0, out[1], out[2], out[3] * z0],
                "A" => [out[0], out[1] * z0, out[2] / z0, out[3]],
                _ => out,
            }
        } else {
            out
        })
    }
}
pub fn noise_circle(nf_db: f64, fmin_db: f64, opt: C, rn_normal: f64) -> Option<(C, f64)> {
    if [nf_db, fmin_db, opt.re, opt.im, rn_normal]
        .iter()
        .any(|v| !v.is_finite())
        || opt.norm() > 1.
    {
        return None;
    }
    if nf_db < fmin_db || rn_normal <= 0. {
        return None;
    }
    let n = (10f64.powf(nf_db / 10.) - 10f64.powf(fmin_db / 10.)) * (1. + opt).norm_sqr()
        / (4. * rn_normal);
    Some((
        opt / (1. + n),
        (n * (n + 1. - opt.norm_sqr())).max(0.).sqrt() / (1. + n),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn close(a: C, b: C) {
        assert!((a - b).norm() < 1e-8, "{a} != {b}");
    }
    #[test]
    fn smith_roundtrip() {
        for z in [c(50., 0.), c(25., -80.), c(200., 100.)] {
            close(impedance(gamma(z, 50.), 50.), z);
        }
    }
    #[test]
    fn quarter_wave() {
        let mut e = Element::new(Kind::Line, 1e9);
        e.value = 0.25;
        close(e.apply(c(25., 0.), 1e9), c(100., 0.));
    }
    #[test]
    fn half_wave_lossless() {
        let mut e = Element::new(Kind::Line, 1e9);
        e.value = 0.5;
        close(e.apply(c(25., 15.), 1e9), c(25., 15.));
    }
    #[test]
    fn l_match() {
        let mut l = Element::new(Kind::SeriesL, 1e9);
        l.value = 25. / TAU;
        let mut cap = Element::new(Kind::ShuntC, 1e9);
        cap.value = 20. / TAU;
        close(transform(c(25., 0.), 1e9, &[l, cap]), c(50., 0.));
    }
    #[test]
    fn shunt_and_transformer() {
        let e = Element::new(Kind::ShuntR, 1e9);
        close(e.apply(c(50., 0.), 1e9), c(25., 0.));
        let mut e = Element::new(Kind::Transformer, 1e9);
        e.value = 2.;
        close(e.apply(c(25., 10.), 1e9), c(100., 40.));
    }
    #[test]
    fn passive_line_loss() {
        let mut e = Element::new(Kind::Line, 1e9);
        e.loss = 20.;
        assert!(gamma(e.apply(c(10., 0.), 1e9), 50.).norm() < gamma(c(10., 0.), 50.).norm());
    }
    #[test]
    fn noise_minimum() {
        let opt = c(0.3, 0.2);
        let (center, r) = noise_circle(1., 1., opt, 0.1).unwrap();
        close(center, opt);
        assert_eq!(r, 0.);
    }
    #[test]
    fn original_amplifier_metrics() {
        let p = SPoint {
            hz: 2e9,
            s: [
                c(-0.103717245, -0.0623196065),
                c(0.7231125, 3.68573523),
                c(0.0549764745, 0.09296014),
                c(0.2380882, -0.333787352),
            ],
        };
        let st = p.stability();
        assert!((st.k - 1.1660794).abs() < 1e-6);
        assert!((st.gain_db - 12.94344624).abs() < 1e-5);
        assert!(p.gain_circle(true, 12.0, false).is_some());
        assert!(p.gain_circle(true, 30.0, false).is_none());
    }
    #[test]
    fn twoport_conversion() {
        let p = SPoint {
            hz: 1e9,
            s: [c(0., 0.), c(0.5, 0.), c(0.5, 0.), c(0., 0.)],
        };
        let z = p.to_parameters("Z", 50., false).unwrap();
        close(z[0], c(250. / 3., 0.));
        close(z[1], c(200. / 3., 0.));
        assert!(p.stability().stable);
        assert!((p.stability().gain_db + 6.020599913).abs() < 1e-6);
    }
}
