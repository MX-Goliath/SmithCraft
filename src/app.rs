use crate::{chart, formats, project::*, rf::*};
use eframe::egui::{self, Color32, Key, Vec2};
use std::{f64::consts::TAU, path::PathBuf};
mod classic;
enum Navigation {
    New,
    Open(PathBuf),
    Close,
}

pub struct SmithApp {
    classic: classic::ClassicState,
    import_dialog: Option<usize>,
    import_selection: Vec<bool>,
    p: Project,
    history: History,
    path: Option<PathBuf>,
    saved: Project,
    status: String,
    mode: bool,
    zoom: f32,
    mouse: bool,
    cursor: Option<C>,
    point_dialog: bool,
    point_frequency_only: bool,
    point_edit: Option<usize>,
    point_mode: usize,
    point_polar: bool,
    point_values: [f64; 3],
    element_dialog: bool,
    element_edit: Option<usize>,
    draft: Element,
    pending: Option<Element>,
    settings: bool,
    circle_dialog: bool,
    sweep_dialog: bool,
    tune: bool,
    tune_centers: Vec<f64>,
    tune_z0_centers: Vec<f64>,
    tune_lc_centers: Vec<[f64; 2]>,
    help: bool,
    circle_kind: usize,
    circle_value: f64,
    circle_source: bool,
    circle_s: [C; 4],
    noise_values: [f64; 5],
    selected_s: usize,
    export_kind: String,
    normalized: bool,
    chart_rect: Option<egui::Rect>,
    copy_rect: Option<egui::Rect>,
    closing: bool,
    navigation: Option<Navigation>,
}
impl SmithApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        classic::setup(&cc.egui_ctx);
        let mut p = Project::default();
        if let Some(previous) = cc
            .storage
            .and_then(|s| eframe::get_value::<Project>(s, "project"))
        {
            p.z0 = previous.z0;
            p.settings = previous.settings;
            p.default_hz = previous.default_hz;
        }
        let mut classic: classic::ClassicState = cc
            .storage
            .and_then(|s| eframe::get_value(s, "classic_ui_v1"))
            .unwrap_or_default();
        classic.upgrade();
        classic.reset_transient();
        let mut a = Self {
            classic,
            import_dialog: None,
            import_selection: vec![],
            saved: p.clone(),
            p,
            history: History::default(),
            path: None,
            status: String::new(),
            mode: false,
            zoom: 1.,
            mouse: false,
            cursor: None,
            point_dialog: false,
            point_frequency_only: false,
            point_edit: None,
            point_mode: 0,
            point_polar: false,
            point_values: [50., 0., 500.],
            element_dialog: false,
            element_edit: None,
            draft: Element::new(Kind::SeriesL, 5e8),
            pending: None,
            settings: false,
            circle_dialog: false,
            sweep_dialog: false,
            tune: false,
            tune_centers: vec![],
            tune_z0_centers: vec![],
            tune_lc_centers: vec![],
            help: false,
            circle_kind: 0,
            circle_value: 2.,
            circle_source: true,
            circle_s: [c(0.3, 0.1), c(2., 0.5), c(0.03, 0.01), c(0.4, -0.1)],
            noise_values: [2., 1., 0.3, 30., 0.2],
            selected_s: 0,
            export_kind: "S".into(),
            normalized: false,
            chart_rect: None,
            copy_rect: None,
            closing: false,
            navigation: None,
        };
        a.mode = a.classic.default_splot;
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.iter().any(|a| a == "--demo") {
            a.p = Project::demo();
            a.saved = a.p.clone();
        } else if let Some(path) = args.first() {
            a.open_path(PathBuf::from(path));
        }
        a
    }
    fn checkpoint(&mut self) {
        self.history.record(&self.p);
    }
    fn modified(&self) -> bool {
        self.p.points != self.saved.points
            || self.p.elements != self.saved.elements
            || self.p.circles != self.saved.circles
            || self.p.sweep != self.saved.sweep
    }
    fn finish_navigation(&mut self, ctx: &egui::Context) {
        self.closing = false;
        match self.navigation.take() {
            Some(Navigation::New) => self.reset_project(),
            Some(Navigation::Open(path)) => self.open_path(path),
            Some(Navigation::Close) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            None => {}
        }
    }
    fn result(&mut self, r: Result<(), String>, success: &str) {
        self.status = match r {
            Ok(()) => success.into(),
            Err(e) => format!("Error: {e}"),
        };
    }
    fn open_path(&mut self, path: PathBuf) {
        let path = path.canonicalize().unwrap_or(path);
        // Keep the displayed Windows path readable (without the extended prefix).
        #[cfg(windows)]
        let path = PathBuf::from(path.to_string_lossy().trim_start_matches(r"\\?\"));
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if matches!(ext.as_str(), "smithcraft" | "json" | "xmlsc") {
            if self.modified() {
                self.navigation = Some(Navigation::Open(path));
                self.closing = true;
                return;
            }
            match formats::load_project_with_reference(&path, self.p.z0) {
                Ok(mut p) => {
                    if ext == "xmlsc" {
                        p.settings = self.p.settings.clone();
                    }
                    self.classic.flags[1] = p.settings.z_grid;
                    self.classic.flags[2] = p.settings.y_grid;
                    self.classic.flags[4] = p.settings.labels;
                    self.classic.flags[9] = p.settings.expanded;
                    self.history.clear();
                    self.p = p;
                    self.saved = self.p.clone();
                    self.remember_file(&path);
                    self.path = Some(path);
                    self.status = "Project opened".into();
                    self.classic.selected_datapoints = vec![self.p.start];
                    self.classic.selected_circles.clear();
                }
                Err(e) => self.status = format!("Error: {e}"),
            }
        } else {
            match formats::read_network(&path) {
                Ok(n) => {
                    self.p.network = Some(n);
                    self.selected_s = 0;
                    self.mode = true;
                    self.remember_file(&path);
                    self.status = "S-parameter data loaded".into();
                }
                Err(e) => self.status = format!("Error: {e}"),
            }
        }
    }
    fn remember_file(&mut self, path: &std::path::Path) {
        self.classic.recent.retain(|p| p != path);
        self.classic.recent.insert(0, path.to_path_buf());
        self.classic.recent.truncate(10);
    }
    fn open(&mut self) {
        let picker = if self.mode {
            rfd::FileDialog::new()
                .add_filter("S-parameter files", &["s1p", "s2p", "cti", "citi", "txt"])
        } else {
            rfd::FileDialog::new()
                .add_filter("Smith XML-files", &["xmlsc"])
                .add_filter("SmithCraft project", &["smithcraft", "json"])
        };
        if let Some(path) = picker.pick_file() {
            self.open_path(path);
        }
    }
    fn save_as(&mut self, force: bool) {
        let path = if force || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("Smith XML-files", &["xmlsc"])
                .add_filter("SmithCraft project", &["smithcraft"])
                .set_file_name(format!("{}.xmlsc", self.p.title))
                .save_file()
        } else {
            self.path.clone()
        };
        if let Some(path) = path {
            let r = formats::save_project(&path, &self.p);
            if r.is_ok() {
                self.remember_file(&path);
                self.path = Some(path);
                self.saved = self.p.clone();
            }
            self.result(r, "Project saved");
        }
    }
    fn export(&mut self, ext: &str, text: String) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(ext, &[ext])
            .set_file_name(format!("{}.{}", self.p.title, ext))
            .save_file()
        {
            self.result(
                std::fs::write(path, text).map_err(|e| e.to_string()),
                "Export complete",
            );
        }
    }
    fn add_point_dialog(&mut self, index: Option<usize>) {
        self.point_frequency_only = false;
        self.classic.fields.clear();
        self.classic.focus_number = Some("point_re".into());
        for id in ["point_re", "point_im"] {
            if index.is_none() {
                self.classic.blank_fields.insert(id.into());
            } else {
                self.classic.blank_fields.remove(id);
            }
        }
        self.point_edit = index;
        self.point_mode = 0;
        self.point_polar = index.is_none();
        self.point_values = if let Some(i) = index {
            let d = &self.p.points[i];
            [d.z.re, d.z.im, d.hz / 1e6]
        } else {
            [0., 0., self.p.default_hz / 1e6]
        };
        self.point_dialog = true;
    }
    fn element(&mut self, kind: Kind, index: Option<usize>) {
        if self.p.active().is_none() {
            self.status = "Add a datapoint before inserting an element".into();
            return;
        }
        self.element_edit = index;
        // Selecting another tool replaces its uncommitted preview, including
        // tools which open a parameter dialog before insertion.
        self.pending = None;
        self.draft = index
            .map(|i| self.p.elements[i].clone())
            .unwrap_or_else(|| Element::new(kind, self.p.active().unwrap().hz));
        self.classic.element_backup = None;
        if index.is_none()
            && !matches!(
                kind,
                Kind::Line
                    | Kind::OpenStub
                    | Kind::ShortStub
                    | Kind::SeriesParallelRlc
                    | Kind::ShuntSeriesRlc
            )
        {
            self.pending = Some(self.draft.clone());
            self.element_dialog = false;
        } else {
            self.element_dialog = true;
        }
        self.mouse = false;
    }
    fn choose_import(&mut self, port: usize) {
        self.import_dialog = Some(port);
        if let Some(n) = &self.p.network {
            if port == 1 && n.ports < 2 {
                self.status = "S22 requires a two-port network".into();
                return;
            }
            self.import_selection = vec![true; n.points.len()];
            self.import_dialog = Some(port);
        }
    }
    fn open_network_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Snp files", &["s1p", "s2p"])
            .add_filter("CITI files", &["citi", "cti"])
            .add_filter("EZNEC files", &["txt"])
            .pick_file()
        {
            match formats::read_network(&path) {
                Ok(n) => {
                    self.import_selection = vec![false; n.points.len()];
                    self.selected_s = 0;
                    self.p.network = Some(n);
                    self.remember_file(&path);
                }
                Err(e) => self.status = e,
            }
        }
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.has_modal() {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.cancel_dialog();
            }
            return;
        }
        let action = ctx.input_mut(|i| {
            let mut a = Vec::new();
            for (k, v) in [
                (Key::N, 0),
                (Key::O, 1),
                (Key::S, 2),
                (Key::Z, 3),
                (Key::Y, 4),
                (Key::M, 5),
                (Key::T, 6),
                (Key::E, 7),
                (Key::Num1, 8),
                (Key::Num2, 9),
                (Key::Num3, 10),
            ] {
                if i.consume_key(egui::Modifiers::CTRL, k) {
                    a.push(v);
                }
            }
            if i.key_pressed(Key::Escape) {
                a.push(11);
            }
            if i.key_pressed(Key::F1) {
                a.push(12);
            }
            a
        });
        for a in action {
            match a {
                0 => self.new_project(),
                1 => self.open(),
                2 => self.save_as(false),
                3 => self.history.undo(&mut self.p),
                4 => self.history.redo(&mut self.p),
                5 => self.mode = !self.mode,
                6 => self.circle_dialog = true,
                7 => self.show_settings(),
                8 => self.zoom = (self.zoom * 1.25).min(8.),
                9 => self.zoom = (self.zoom / 1.25).max(0.1),
                10 => self.zoom = 1.,
                11 => {
                    self.pending = None;
                    self.mouse = false;
                    self.point_dialog = false;
                    self.element_dialog = false;
                }
                12 => {
                    self.classic.help_topic = 0;
                    self.help = true;
                }
                _ => {}
            }
        }
    }
    fn draw_circle(&mut self) {
        let sp = SPoint {
            hz: 1.,
            s: self.circle_s,
        };
        let mut q = None;
        let value = self.circle_value;
        let result = match self.circle_kind {
            0 if value >= 1. => Some((
                c(0., 0.),
                (value - 1.) / (value + 1.),
                format!("VSWR = {value:.3}"),
            )),
            1 if value >= 0. => {
                q = Some(value);
                Some((c(0., 0.), 0., format!("Q = {value:.3}")))
            }
            2 => sp.stability_circle(self.circle_source).map(|(center, r)| {
                let s = if self.circle_source {
                    self.circle_s[3]
                } else {
                    self.circle_s[0]
                };
                let origin_inside = center.norm() < r;
                let origin_stable = s.norm() < 1.;
                (
                    center,
                    r,
                    format!(
                        "Stability {} — stable {}",
                        if self.circle_source {
                            "input"
                        } else {
                            "output"
                        },
                        if origin_inside == origin_stable {
                            "inside"
                        } else {
                            "outside"
                        }
                    ),
                )
            }),
            3..=5 => {
                let source = if self.circle_kind == 3 {
                    true
                } else if self.circle_kind == 4 {
                    false
                } else {
                    self.circle_source
                };
                sp.gain_circle(source, value, self.circle_kind == 5)
                    .map(|(center, r)| {
                        (
                            center,
                            r,
                            format!(
                                "Gain {} {value:.3} dB",
                                if source { "input" } else { "output" }
                            ),
                        )
                    })
            }
            6 => {
                let [nf, fmin, mag, phase, rn] = self.noise_values;
                noise_circle(nf, fmin, C::from_polar(mag, phase.to_radians()), rn)
                    .map(|(center, r)| (center, r, format!("Noise NF={nf:.3} dB")))
            }
            _ => None,
        };
        if let Some((center, radius, label)) = result {
            self.p.circles.push(Circle {
                definition: match self.circle_kind {
                    2 => Some(CircleDefinition::Stability {
                        hz: self.classic.circle_frequency.unwrap_or(self.p.default_hz),
                        s: self.circle_s,
                        source: self.circle_source,
                    }),
                    3..=5 => Some(CircleDefinition::Gain {
                        hz: self.classic.circle_frequency.unwrap_or(self.p.default_hz),
                        s: self.circle_s,
                        source: if self.circle_kind == 3 {
                            true
                        } else if self.circle_kind == 4 {
                            false
                        } else {
                            self.circle_source
                        },
                        unilateral: self.circle_kind == 5,
                        gain_db: value,
                    }),
                    6 => Some(CircleDefinition::Noise {
                        hz: self.classic.circle_frequency.unwrap_or(self.p.default_hz),
                        nf_db: self.noise_values[0],
                        minimum_db: self.noise_values[1],
                        optimum: C::from_polar(
                            self.noise_values[2],
                            self.noise_values[3].to_radians(),
                        ),
                        rn: self.noise_values[4],
                    }),
                    _ => None,
                },
                details: match self.circle_kind {
                    2..=5 => format!(
                        "S11={} ; S21={} ; S12={} ; S22={} ; K={:.6} ; MAG/MSG={:.6} dB",
                        self.circle_s[0],
                        self.circle_s[1],
                        self.circle_s[2],
                        self.circle_s[3],
                        sp.stability().k,
                        sp.stability().gain_db
                    ),
                    6 => format!(
                        "NF={} dB; NFmin={} dB; GammaOpt={} at {} degrees; Rn/Z0={}",
                        self.noise_values[0],
                        self.noise_values[1],
                        self.noise_values[2],
                        self.noise_values[3],
                        self.noise_values[4]
                    ),
                    _ => label.clone(),
                },
                center,
                radius,
                label,
                visible: true,
                highlighted: false,
                q,
            });
            self.status = "Circle added".into();
        } else {
            self.status =
                "No real circle for these values; check gain / noise / VSWR limits".into();
        }
    }
}

impl eframe::App for SmithApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && self.modified() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.closing = true;
            self.navigation = Some(Navigation::Close);
        }
        let events = ctx.input(|i| i.events.clone());
        for e in events {
            if let egui::Event::Screenshot { image, .. } = e
                && let Some(rect) = self.copy_rect.take()
            {
                let scale = ctx.pixels_per_point();
                let minx = (rect.min.x * scale).max(0.) as usize;
                let miny = (rect.min.y * scale).max(0.) as usize;
                let maxx = ((rect.max.x * scale) as usize).min(image.size[0]);
                let maxy = ((rect.max.y * scale) as usize).min(image.size[1]);
                if maxx > minx && maxy > miny {
                    let mut pixels = Vec::new();
                    for y in miny..maxy {
                        pixels.extend_from_slice(
                            &image.pixels[y * image.size[0] + minx..y * image.size[0] + maxx],
                        );
                    }
                    ctx.copy_image(egui::ColorImage::new([maxx - minx, maxy - miny], pixels));
                    self.status = "Chart image copied".into();
                }
            }
        }
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for f in dropped {
            if let Some(path) = f.path {
                self.open_path(path);
            }
        }
        self.shortcuts(ctx);
        self.classic_menus(ctx);
        self.classic_workspace(ctx);
        self.classic_dialogs(ctx);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "project", &self.p);
        let mut classic = self.classic.clone();
        classic.reset_transient();
        eframe::set_value(storage, "classic_ui_v1", &classic);
    }
}
