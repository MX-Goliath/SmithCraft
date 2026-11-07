//! Constrained graphical insertion. Component values parameterize a Mobius
//! circle, so the pointer can be projected onto the *reachable* arc exactly.
use crate::rf::*;
use std::f64::consts::TAU;

#[derive(Clone, Copy)]
enum Curve {
    Arc {
        center: C,
        radius: f64,
        angle: f64,
        sweep: f64,
    },
    Segment(C, C),
    Point(C),
}
impl Curve {
    fn project(self, target: C, fallback: C) -> C {
        match self {
            Self::Arc {
                center,
                radius,
                angle,
                sweep,
            } => {
                let offset = target - center;
                if offset.norm() < 1e-14 {
                    return fallback;
                }
                let phase = (offset.arg() - angle).rem_euclid(TAU);
                let allowed = if sweep >= 0. {
                    phase <= sweep + 1e-12
                } else {
                    (TAU - phase).rem_euclid(TAU) <= -sweep + 1e-12
                };
                if allowed {
                    center + offset / offset.norm() * radius
                } else {
                    let a = center + C::from_polar(radius, angle);
                    let b = center + C::from_polar(radius, angle + sweep);
                    if (target - a).norm_sqr() <= (target - b).norm_sqr() {
                        a
                    } else {
                        b
                    }
                }
            }
            Self::Segment(a, b) => {
                let d = b - a;
                let t = (((target - a) * d.conj()).re / d.norm_sqr().max(1e-30)).clamp(0., 1.);
                a + d * t
            }
            Self::Point(p) => p,
        }
    }
    fn points(self) -> Vec<C> {
        match self {
            Self::Arc {
                center,
                radius,
                angle,
                sweep,
            } => (0..=360)
                .map(|i| center + C::from_polar(radius, angle + sweep * i as f64 / 360.))
                .collect(),
            Self::Segment(a, b) => vec![a, b],
            Self::Point(p) => vec![p],
        }
    }
}

/// The circle through three reflection coefficients, computed in local
/// coordinates to retain precision for small loci near an open/short circuit.
fn circle(a: C, b: C, d: C) -> Option<(C, f64)> {
    let scale = (b - a).norm().max((d - a).norm());
    if scale < 1e-14 {
        return None;
    }
    let m = (b - a) / scale;
    let n = (d - a) / scale;
    let det = m.re * n.im - m.im * n.re;
    if det.abs() < 1e-12 * m.norm() * n.norm() {
        return None;
    }
    let center = c(
        (m.norm_sqr() * n.im - n.norm_sqr() * m.im) / (2. * det),
        (m.re * n.norm_sqr() - n.re * m.norm_sqr()) / (2. * det),
    );
    Some((a + center * scale, center.norm() * scale))
}

struct ParameterMap {
    origin: C,
    scale: f64,
    a: C,
    c: C,
    value_scale: f64,
    squared: bool,
}
pub struct Locus {
    draft: Element,
    load: C,
    hz: f64,
    z0: f64,
    curve: Curve,
    parameter: Option<ParameterMap>,
}
pub struct Preview {
    pub element: Element,
    pub z: C,
    pub gamma: C,
}
impl Locus {
    pub fn new(draft: &Element, load: C, hz: f64, z0: f64) -> Option<Self> {
        let w = TAU * hz;
        let z_scale = load.norm().max(z0);
        let y_scale = inv(load).norm().max(1. / z0);
        let (value_scale, start, end, squared) = match draft.kind {
            Kind::SeriesR => (z_scale, gamma(load, z0), c(1., 0.), false),
            Kind::SeriesL => (z_scale / w * 1e9, gamma(load, z0), c(1., 0.), false),
            Kind::SeriesC => (1. / (w * z_scale) * 1e12, c(1., 0.), gamma(load, z0), false),
            Kind::ShuntR => (1. / y_scale, c(-1., 0.), gamma(load, z0), false),
            Kind::ShuntL => (1. / (w * y_scale) * 1e9, c(-1., 0.), gamma(load, z0), false),
            Kind::ShuntC => (y_scale / w * 1e12, gamma(load, z0), c(-1., 0.), false),
            Kind::Transformer => (1., c(-1., 0.), c(1., 0.), true),
            Kind::Line | Kind::OpenStub | Kind::ShortStub => {
                return Self::periodic(draft, load, hz, z0);
            }
            _ => return None,
        };
        let mut sample = draft.clone();
        sample.value = if squared {
            value_scale.sqrt()
        } else {
            value_scale
        };
        let middle = gamma(sample.apply(load, hz), z0);
        let scale = (middle - start).norm().max((end - start).norm());
        let (curve, parameter) = if scale < 1e-14 {
            (Curve::Point(start), None)
        } else {
            let m = (middle - start) / scale;
            let d = (end - start) / scale;
            if (m - d).norm() < 1e-14 {
                return None;
            }
            let coef = (d - 2. * m) / (m - d);
            let parameter = ParameterMap {
                origin: start,
                scale,
                a: d * (coef + 1.),
                c: coef,
                value_scale,
                squared,
            };
            let curve = if let Some((center, radius)) = circle(start, middle, end) {
                let angle = (start - center).arg();
                let end_angle = ((end - center).arg() - angle).rem_euclid(TAU);
                let mid_angle = ((middle - center).arg() - angle).rem_euclid(TAU);
                let sweep = if mid_angle <= end_angle {
                    end_angle
                } else {
                    end_angle - TAU
                };
                Curve::Arc {
                    center,
                    radius,
                    angle,
                    sweep,
                }
            } else {
                Curve::Segment(start, end)
            };
            (curve, Some(parameter))
        };
        Some(Self {
            draft: draft.clone(),
            load,
            hz,
            z0,
            curve,
            parameter,
        })
    }
    fn periodic(draft: &Element, load: C, hz: f64, z0: f64) -> Option<Self> {
        if draft.kind == Kind::Line && draft.loss > 0. {
            return None;
        }
        let lengths = match draft.kind {
            Kind::Line => [0., 0.125, 0.25],
            Kind::OpenStub => [0., 0.125, 0.375],
            _ => [0.125, 0.25, 0.375],
        };
        let points = lengths.map(|v| {
            let mut e = draft.clone();
            e.value = v * draft.reference_hz / hz;
            gamma(e.apply(load, hz), z0)
        });
        let curve = if let Some((center, radius)) = circle(points[0], points[1], points[2]) {
            Curve::Arc {
                center,
                radius,
                angle: 0.,
                sweep: TAU,
            }
        } else {
            Curve::Point(points[0])
        };
        Some(Self {
            draft: draft.clone(),
            load,
            hz,
            z0,
            curve,
            parameter: None,
        })
    }
    pub fn points(&self) -> Vec<C> {
        self.curve.points()
    }
    pub fn preview(&self, target: C) -> Preview {
        let mut element = self.draft.clone();
        let fallback = gamma(element.apply(self.load, self.hz), self.z0);
        let g = self.curve.project(target, fallback);
        let value = if let Some(map) = &self.parameter {
            let y = (g - map.origin) / map.scale;
            let den = map.a - y * map.c;
            let t = if den.norm() < 1e-14 { 1. } else { (y / den).re };
            let t = t.clamp(1e-9, 1. - 1e-9);
            let u = map.value_scale * t / (1. - t);
            if map.squared { u.sqrt() } else { u }
        } else if matches!(self.curve, Curve::Point(_)) {
            element.value
        } else {
            let z = impedance(g, self.z0);
            let b = (inv(z) - inv(self.load)).im;
            let length = match element.kind {
                Kind::Line => (-(gamma(z, element.z0) / gamma(self.load, element.z0)).arg()
                    / (2. * TAU))
                    .rem_euclid(0.5),
                Kind::OpenStub if (g + c(1., 0.)).norm() < 1e-10 => 0.25,
                Kind::ShortStub if (g + c(1., 0.)).norm() < 1e-10 => 1e-9,
                Kind::OpenStub => (b * element.z0).atan().rem_euclid(std::f64::consts::PI) / TAU,
                _ => {
                    (-1. / (b * element.z0))
                        .atan()
                        .rem_euclid(std::f64::consts::PI)
                        / TAU
                }
            };
            length.max(1e-9) * element.reference_hz / self.hz
        };
        if value.is_finite() && value > 0. {
            element.value = value;
        }
        let z = element.apply(self.load, self.hz);
        Preview {
            element,
            z,
            gamma: gamma(z, self.z0),
        }
    }
}

/// Engineering units used beside the insertion cursor and in the schematic.
pub fn value_label(e: &Element) -> String {
    let (value, units): (f64, &[(f64, &str)]) = match e.kind {
        Kind::SeriesL | Kind::ShuntL => (
            e.value * 1e-9,
            &[
                (1., "H"),
                (1e-3, "mH"),
                (1e-6, "µH"),
                (1e-9, "nH"),
                (1e-12, "pH"),
            ],
        ),
        Kind::SeriesC | Kind::ShuntC => (
            e.value * 1e-12,
            &[
                (1., "F"),
                (1e-3, "mF"),
                (1e-6, "µF"),
                (1e-9, "nF"),
                (1e-12, "pF"),
                (1e-15, "fF"),
            ],
        ),
        Kind::SeriesR | Kind::ShuntR => (
            e.value,
            &[
                (1e9, "GΩ"),
                (1e6, "MΩ"),
                (1e3, "kΩ"),
                (1., "Ω"),
                (1e-3, "mΩ"),
            ],
        ),
        Kind::Transformer => return format!("{:.2}:1", e.value),
        _ => return format!("{:.2}λ", e.value),
    };
    let (scale, unit) = units
        .iter()
        .find(|(scale, _)| value.abs() >= *scale)
        .unwrap_or(units.last().unwrap());
    format!("{:.2}{unit}", value / scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointer_projects_to_reachable_positive_component_arc() {
        for kind in [
            Kind::SeriesR,
            Kind::ShuntR,
            Kind::SeriesL,
            Kind::ShuntL,
            Kind::SeriesC,
            Kind::ShuntC,
            Kind::Transformer,
        ] {
            for load in [c(32., 17.), c(75., -120.), c(25., -3000.), c(50., 0.)] {
                let draft = Element::new(kind, 5e8);
                let locus = Locus::new(&draft, load, 5e8, 75.).unwrap();
                let samples: Vec<_> = (0..=2000)
                    .map(|i| {
                        let mut e = draft.clone();
                        e.value *= 10f64.powf(-8. + 16. * i as f64 / 2000.);
                        gamma(e.apply(load, 5e8), 75.)
                    })
                    .collect();
                for target in [
                    c(-0.8, 0.7),
                    c(0.7, -0.8),
                    c(0., 0.),
                    c(0.4, 0.6),
                    c(-1.2, -0.4),
                ] {
                    let p = locus.preview(target);
                    assert!(p.element.value > 0. && p.element.value.is_finite());
                    assert!((p.gamma - gamma(p.element.apply(load, 5e8), 75.)).norm() < 1e-12);
                    let sampled = samples
                        .iter()
                        .map(|g| (*g - target).norm())
                        .fold(f64::INFINITY, f64::min);
                    assert!(
                        (p.gamma - target).norm() <= sampled + 1e-6,
                        "{kind:?}, load={load}, target={target}, got={}, sampled={sampled}",
                        (p.gamma - target).norm()
                    );
                    if kind == Kind::SeriesL {
                        assert!((p.z.re - load.re).abs() < 1e-8 && p.z.im >= load.im);
                    }
                    if kind == Kind::SeriesC {
                        assert!((p.z.re - load.re).abs() < 1e-8 && p.z.im <= load.im);
                    }
                }
            }
        }
    }
    #[test]
    fn periodic_projection_uses_line_impedance_and_reference_frequency() {
        for kind in [Kind::Line, Kind::OpenStub, Kind::ShortStub] {
            let mut draft = Element::new(kind, 2e9);
            draft.z0 = 67.;
            let load = c(32., 17.);
            let hz = 7e8;
            let locus = Locus::new(&draft, load, hz, 75.).unwrap();
            for wavelength in [0.01, 0.1, 0.22, 0.33, 0.49] {
                let mut expected = draft.clone();
                expected.value = wavelength * draft.reference_hz / hz;
                let g = gamma(expected.apply(load, hz), 75.);
                let p = locus.preview(g);
                assert!(
                    (p.gamma - g).norm() < 1e-7,
                    "{kind:?}: {wavelength}, {}",
                    (p.gamma - g).norm()
                );
            }
        }
    }
}
