use crate::rf::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Datapoint {
    pub name: String,
    pub z: C,
    pub hz: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Circle {
    pub label: String,
    #[serde(default)]
    pub details: String,
    pub center: C,
    pub radius: f64,
    pub visible: bool,
    pub highlighted: bool,
    pub q: Option<f64>,
    #[serde(default)]
    pub definition: Option<CircleDefinition>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircleDefinition {
    Stability {
        hz: f64,
        s: [C; 4],
        source: bool,
    },
    Gain {
        hz: f64,
        s: [C; 4],
        source: bool,
        unilateral: bool,
        gain_db: f64,
    },
    Noise {
        hz: f64,
        nf_db: f64,
        minimum_db: f64,
        optimum: C,
        rn: f64,
    },
}
impl Circle {
    pub fn family(&self) -> char {
        if self.q.is_some() {
            'Q'
        } else if self.label.starts_with("VSWR") {
            'V'
        } else if self.label.starts_with("Gain") {
            'G'
        } else if self.label.starts_with("Noise") {
            'N'
        } else {
            'S'
        }
    }
    pub fn description(&self, name: &str) -> String {
        let frequency = |hz: f64| {
            if hz >= 1e9 {
                format!("{:.2}GHz", hz / 1e9)
            } else {
                format!("{:.2}MHz", hz / 1e6)
            }
        };
        let params = |s: &[C; 4]| {
            [(0, "S11"), (2, "S12"), (1, "S21"), (3, "S22")]
                .iter()
                .map(|(i, name)| {
                    format!(
                        "{name}={:.2} < {:.2}°",
                        s[*i].norm(),
                        s[*i].arg().to_degrees()
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        match &self.definition {
            Some(CircleDefinition::Gain {
                hz,
                s,
                source,
                unilateral,
                gain_db,
            }) => format!(
                "{name}: {} plane {} gain circle ;Vp={gain_db:.2}dB ;Gmax={:.2}dB ;{}; {}",
                if *source { "input" } else { "output" },
                if *unilateral { "unilateral" } else { "const." },
                SPoint { hz: *hz, s: *s }.stability().gain_db,
                params(s),
                frequency(*hz)
            ),
            Some(CircleDefinition::Stability { hz, s, source }) => {
                let stable = if *source {
                    s[3].norm() < 1.
                } else {
                    s[0].norm() < 1.
                };
                format!(
                    "{name}: {} plane stability circle; stable {}; K={:.2}; {}; {}",
                    if *source { "Input" } else { "Output" },
                    if stable == (self.center.norm() < self.radius) {
                        "inside"
                    } else {
                        "outside"
                    },
                    SPoint { hz: *hz, s: *s }.stability().k,
                    params(s),
                    frequency(*hz)
                )
            }
            Some(CircleDefinition::Noise {
                hz,
                nf_db,
                minimum_db,
                optimum,
                rn,
            }) => format!(
                "{name}: Constant {nf_db:.2}dB noise figure circle; NFmin = {minimum_db:.2}dB; Γ_NFmin = {:.2} < {:.2}; r'noise = {rn:.2};{}",
                optimum.norm(),
                optimum.arg().to_degrees(),
                frequency(*hz)
            ),
            None => {
                let label = if let Some(q) = self.q {
                    format!("Q = {q:.2}")
                } else if let Some(vswr) = self
                    .label
                    .strip_prefix("VSWR = ")
                    .and_then(|value| value.parse::<f64>().ok())
                {
                    format!("VSWR = {vswr:.2}")
                } else {
                    self.label.clone()
                };
                format!("{name}: {label}; {}", self.details)
            }
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sweep {
    pub start: f64,
    pub stop: f64,
    pub count: usize,
    pub logarithmic: bool,
    pub datapoints: bool,
    pub visible: bool,
    #[serde(default)]
    pub step_hz: Option<f64>,
}
impl Default for Sweep {
    fn default() -> Self {
        Self {
            start: 1e8,
            stop: 5e8,
            count: 9,
            logarithmic: false,
            datapoints: true,
            visible: false,
            step_hz: Some(5e7),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(default)]
    pub expanded: bool,
    pub z_grid: bool,
    pub y_grid: bool,
    pub labels: bool,
    pub grid_width: f32,
    pub trace_width: f32,
    pub z_color: [u8; 3],
    pub y_color: [u8; 3],
    pub trace_color: [u8; 3],
    pub vertical: bool,
    pub log_frequency: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            expanded: false,
            z_grid: true,
            y_grid: true,
            labels: true,
            grid_width: 0.7,
            trace_width: 2.,
            z_color: [210, 45, 50],
            y_color: [15, 139, 198],
            trace_color: [40, 90, 195],
            vertical: false,
            log_frequency: false,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Project {
    pub title: String,
    pub z0: f64,
    pub default_hz: f64,
    pub points: Vec<Datapoint>,
    pub start: usize,
    pub elements: Vec<Element>,
    pub circles: Vec<Circle>,
    pub sweep: Sweep,
    pub settings: Settings,
    pub network: Option<crate::formats::Network>,
}
impl Default for Project {
    fn default() -> Self {
        Self {
            title: "Untitled".into(),
            z0: 50.,
            default_hz: 5e8,
            points: vec![],
            start: 0,
            elements: vec![],
            circles: vec![],
            sweep: Sweep::default(),
            settings: Settings::default(),
            network: None,
        }
    }
}
impl Project {
    pub fn circle_name(&self, index: usize) -> String {
        let family = self.circles[index].family();
        if matches!(family, 'Q' | 'V') {
            return self.circles[index].label.clone();
        }
        format!(
            "{family}{}",
            self.circles[..=index]
                .iter()
                .filter(|c| c.family() == family)
                .count()
        )
    }
    pub fn active(&self) -> Option<&Datapoint> {
        self.points.get(self.start)
    }
    pub fn stages(&self) -> Vec<C> {
        let Some(p) = self.active() else {
            return vec![];
        };
        let mut result = vec![p.z];
        for e in &self.elements {
            result.push(e.apply(*result.last().unwrap(), p.hz));
        }
        result
    }
    pub fn swept(&self) -> Vec<(f64, C)> {
        if !self.sweep.visible {
            return vec![];
        }
        if self.sweep.datapoints {
            return self
                .points
                .iter()
                .map(|p| (p.hz, transform(p.z, p.hz, &self.elements)))
                .collect();
        }
        let Some(p) = self.active() else {
            return vec![];
        };
        let sw = &self.sweep;
        (0..sw.count.clamp(2, 10001))
            .map(|i| {
                let t = i as f64 / (sw.count.clamp(2, 10001) - 1) as f64;
                let f = if let Some(step) = sw.step_hz {
                    sw.start + i as f64 * step
                } else if sw.logarithmic {
                    sw.start * (sw.stop / sw.start).powf(t)
                } else {
                    sw.start + (sw.stop - sw.start) * t
                };
                (f, transform(p.z, f, &self.elements))
            })
            .collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.z0.is_finite()
            || self.z0 <= 0.
            || !self.default_hz.is_finite()
            || self.default_hz <= 0.
        {
            return Err("Reference impedance and frequency must be positive and finite".into());
        }
        if self
            .points
            .iter()
            .any(|p| !p.z.re.is_finite() || !p.z.im.is_finite() || !p.hz.is_finite() || p.hz <= 0.)
        {
            return Err("Invalid datapoint".into());
        }
        if self.elements.iter().any(|e| {
            [e.value, e.l_nh, e.c_pf, e.z0, e.er, e.reference_hz]
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.)
                || !e.loss.is_finite()
                || e.loss < 0.
        }) {
            return Err("Element values must be positive; attenuation must be nonnegative".into());
        }
        if !self.sweep.start.is_finite()
            || !self.sweep.stop.is_finite()
            || self.sweep.start <= 0.
            || self.sweep.stop <= self.sweep.start
            || self
                .sweep
                .step_hz
                .is_some_and(|s| !s.is_finite() || s <= 0.)
            || !(2..=10001).contains(&self.sweep.count)
        {
            return Err("Sweep requires 0 < start < stop".into());
        }
        if self.circles.iter().any(|x| {
            !x.center.re.is_finite()
                || !x.center.im.is_finite()
                || !x.radius.is_finite()
                || x.radius < 0.
        }) {
            return Err("Invalid circle".into());
        }
        Ok(())
    }
    pub fn demo() -> Self {
        let mut p = Self {
            title: "25 to 50 ohm L-match".into(),
            default_hz: 1e9,
            ..Self::default()
        };
        p.points.push(Datapoint {
            name: "DP1".into(),
            z: c(25., 0.),
            hz: 1e9,
        });
        let mut l = Element::new(Kind::SeriesL, 1e9);
        l.value = 25. / std::f64::consts::TAU;
        let mut cap = Element::new(Kind::ShuntC, 1e9);
        cap.value = 20. / std::f64::consts::TAU;
        p.elements = vec![l, cap];
        p.sweep = Sweep {
            start: 0.5e9,
            stop: 1.5e9,
            count: 101,
            datapoints: false,
            step_hz: None,
            visible: true,
            ..Sweep::default()
        };
        p
    }
}
#[derive(Default)]
pub struct History {
    past: Vec<Project>,
    future: Vec<Project>,
}
impl History {
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
    fn apply(p: &mut Project, snapshot: Project) {
        p.points = snapshot.points;
        p.elements = snapshot.elements;
        p.start = snapshot.start;
        p.sweep = snapshot.sweep;
    }
    pub fn record(&mut self, p: &Project) {
        if self.past.last() != Some(p) {
            self.past.push(p.clone());
            if self.past.len() > 100 {
                self.past.remove(0);
            }
            self.future.clear();
        }
    }
    pub fn undo(&mut self, p: &mut Project) {
        if let Some(old) = self.past.pop() {
            self.future.push(p.clone());
            Self::apply(p, old);
        }
    }
    pub fn redo(&mut self, p: &mut Project) {
        if let Some(new) = self.future.pop() {
            self.past.push(p.clone());
            Self::apply(p, new);
        }
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;
    #[test]
    fn undo_preserves_display_settings_and_circles() {
        let mut p = Project::demo();
        let mut h = History::default();
        h.record(&p);
        let original = p.elements[0].value;
        p.elements[0].value *= 1.2;
        p.settings.z_grid = false;
        p.circles.push(Circle {
            label: "VSWR = 2".into(),
            details: String::new(),
            center: c(0., 0.),
            radius: 1. / 3.,
            visible: true,
            highlighted: false,
            q: None,
            definition: None,
        });
        h.undo(&mut p);
        assert_eq!(p.elements[0].value, original);
        assert!(!p.settings.z_grid);
        assert_eq!(p.circles.len(), 1);
        h.redo(&mut p);
        assert_eq!(p.elements[0].value, original * 1.2);
        assert_eq!(p.circles.len(), 1);
    }
}
