use super::*;
fn footer(ui: &mut Ui, y: f32, width: f32) -> (bool, bool, bool) {
    let x = (width - 168.).max(0.);
    let ok = action(ui, x, y, 52., "OK", 0);
    let cancel = action(ui, x + 54., y, 58., "Cancel", 1);
    let help = action(ui, x + 114., y, 52., "Help", 2);
    (ok, cancel, help)
}
fn action(ui: &mut Ui, x: f32, y: f32, w: f32, text: &str, kind: usize) -> bool {
    let r = at(ui, x, y, w, 19.);
    let response = ui.put(
        r,
        egui::Button::new(egui::RichText::new(format!("  {text}")).size(11.))
            .wrap_mode(egui::TextWrapMode::Extend),
    );
    let p = ui.painter();
    let c = r.left_center() + vec2(8., 0.);
    match kind {
        0 => {
            p.line_segment(
                [c + vec2(-4., 0.), c + vec2(-1., 4.)],
                Stroke::new(2.0_f32, Color32::from_rgb(70, 165, 60)),
            );
            p.line_segment(
                [c + vec2(-1., 4.), c + vec2(5., -5.)],
                Stroke::new(2.0_f32, Color32::from_rgb(70, 165, 60)),
            );
        }
        1 => {
            p.line_segment(
                [c + vec2(-4., -4.), c + vec2(4., 4.)],
                Stroke::new(1.5f32, Color32::from_rgb(190, 75, 65)),
            );
            p.line_segment(
                [c + vec2(-4., 4.), c + vec2(4., -4.)],
                Stroke::new(1.5f32, Color32::from_rgb(190, 75, 65)),
            );
        }
        _ => {
            p.text(
                c,
                Align2::CENTER_CENTER,
                "?",
                FontId::proportional(16.),
                Color32::from_rgb(40, 100, 170),
            );
        }
    }
    response.clicked()
}
#[allow(clippy::too_many_arguments)] // Fixed coordinates follow the reference dialog layout.
fn num(
    ui: &mut Ui,
    s: &mut ClassicState,
    id: &str,
    x: f32,
    y: f32,
    w: f32,
    v: &mut f64,
    on: bool,
) -> bool {
    let r = at(ui, x, y, w, 18.);
    number(ui, s, id, r, v, on)
}
fn tabs(ui: &mut Ui, selected: &mut usize, labels: &[&str]) {
    let padding = ui.spacing().button_padding;
    ui.spacing_mut().button_padding = vec2(2., 0.);
    let mut x = 0.;
    for (i, label) in labels.iter().enumerate() {
        let w = label.len() as f32 * 4. + 5.;
        let r = at(ui, x, 5., w, 14.);
        let response = ui.put(
            r,
            egui::Button::new(egui::RichText::new(*label).size(8.))
                .wrap_mode(egui::TextWrapMode::Extend)
                .min_size(Vec2::ZERO)
                .fill(if *selected == i {
                    FORM
                } else {
                    Color32::from_gray(229)
                }),
        );
        if response.clicked() {
            *selected = i;
        }
        x += w;
    }
    ui.spacing_mut().button_padding = padding;
    ui.painter().line_segment(
        [
            at(ui, 0., 19., ui.available_width(), 0.).left_top(),
            at(ui, 0., 19., ui.available_width(), 0.).right_top(),
        ],
        Stroke::new(1.0_f32, Color32::WHITE),
    );
}
fn tune_slider(ui: &mut Ui, x: f32, value: &mut f64, base: f64, unit: &str) {
    label(ui, x + 7., 42., 70., unit);
    ui.scope_builder(
        UiBuilder::new().max_rect(at(ui, x + 8., 60., 19., 111.)),
        |ui| {
            ui.add(
                egui::Slider::new(value, base * 0.8..=base * 1.2)
                    .vertical()
                    .show_value(false),
            );
        },
    );
    label(ui, x + 35., 60., 43., "Max.");
    field(ui, x + 35., 75., 38., &format!("{:.2}", base * 1.2));
    label(ui, x + 35., 97., 43., "Value");
    field(ui, x + 35., 112., 38., &format!("{value:.2}"));
    label(ui, x + 35., 141., 43., "Min.");
    field(ui, x + 35., 157., 38., &format!("{:.2}", base * 0.8));
}
impl SmithApp {
    fn modal(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        title: &str,
        size: Vec2,
        close_box: bool,
        mut content: impl FnMut(&mut Self, &mut Ui),
    ) -> bool {
        let mut state = ClassicState {
            dialog_positions: std::mem::take(&mut self.classic.dialog_positions),
            ..ClassicState::default()
        };
        let (_, close) = dialog(ctx, &mut state, key, title, size, close_box, |ui| {
            content(self, ui)
        });
        self.classic.dialog_positions = state.dialog_positions;
        if close {
            self.cancel_dialog();
            ctx.request_repaint();
        }
        close
    }
    pub(in super::super) fn classic_dialogs(&mut self, ctx: &egui::Context) {
        if self.has_modal() {
            let r = ctx.content_rect();
            egui::Area::new(Id::new("modal_input_blocker"))
                .fixed_pos(r.min)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.set_min_size(r.size());
                    ui.interact(
                        Rect::from_min_size(r.min, r.size()),
                        Id::new("modal_input_surface"),
                        Sense::click_and_drag(),
                    );
                });
        }
        if let Some(key) = self.classic.capture_pending.take() {
            let d = self.classic.desktop;
            if let Some(w) = self.classic.windows.get(&key) {
                let r = if w.maximized {
                    [0., 0., d[2], d[3]]
                } else {
                    w.rect
                };
                self.copy_rect = Some(Rect::from_min_size(
                    pos2(
                        d[0] + r[0] - self.classic.scroll[0] + 7.,
                        d[1] + r[1] - self.classic.scroll[1] + 29.,
                    ),
                    vec2(r[2] - 14., r[3] - 36.),
                ));
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            }
        }
        if self.point_dialog {
            self.point_form(ctx);
        }
        if self.element_dialog {
            self.element_form(ctx);
        }
        if self.sweep_dialog {
            self.sweep_form(ctx);
        }
        if self.settings {
            self.settings_form(ctx);
        }
        if self.tune {
            self.tune_form(ctx);
        }
        if self.circle_dialog {
            self.circles_form(ctx);
        }
        if self.import_dialog.is_some() {
            self.import_form(ctx);
        }
        if self.classic.copy_dialog {
            self.copy_form(ctx);
        }
        if self.classic.print_dialog {
            self.print_form(ctx);
        }
        if self.classic.preview_open {
            self.preview_form(ctx);
        }
        if self.classic.export_dialog {
            self.export_form(ctx);
        }
        if self.classic.parameter_dialog {
            self.parameter_form(ctx);
        }
        if self.help {
            self.help_form(ctx);
        }
        if self.classic.about {
            self.about_form(ctx);
        }
        if self.closing {
            self.close_form(ctx);
        }
    }
    pub(in super::super) fn has_modal(&self) -> bool {
        self.closing
            || self.point_dialog
            || self.element_dialog
            || self.sweep_dialog
            || self.settings
            || self.circle_dialog
            || self.import_dialog.is_some()
            || self.classic.copy_dialog
            || self.classic.print_dialog
            || self.classic.parameter_dialog
            || self.classic.export_dialog
            || self.classic.about
    }
    pub(in super::super) fn cancel_dialog(&mut self) {
        if self.import_dialog.is_some() {
            self.import_dialog = None;
            self.classic.import_for_circle = false;
        } else if self.help {
            self.help = false;
        } else if self.settings {
            self.cancel_settings();
        } else if self.element_dialog {
            if let Some(p) = self.classic.element_backup.take() {
                self.p = p;
            }
            self.element_dialog = false;
        } else if self.sweep_dialog {
            if let Some(s) = self.classic.sweep_backup.take() {
                self.p.sweep = s;
            }
            self.sweep_dialog = false;
        } else if self.point_dialog {
            self.point_dialog = false;
        } else if self.circle_dialog {
            if let Some(c) = self.classic.circle_backup.take() {
                self.p.circles = c;
            }
            self.circle_dialog = false;
        } else {
            self.classic.copy_dialog = false;
            self.classic.print_dialog = false;
            self.classic.parameter_dialog = false;
            self.classic.export_dialog = false;
            self.classic.about = false;
        }
    }
    fn close_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(
            ctx,
            "close_project",
            "Smith",
            vec2(360., 140.),
            true,
            |a, ui| {
                label(ui, 14., 11., 320., "Save changes to the current project?");
                if button(ui, 95., 64., 70., 23., "Save") {
                    a.save_as(false);
                    if !a.modified() {
                        a.finish_navigation(ctx);
                    }
                }
                if button(ui, 170., 64., 75., 23., "Discard") {
                    a.saved = a.p.clone();
                    a.finish_navigation(ctx);
                }
                if button(ui, 250., 64., 75., 23., "Cancel") {
                    a.closing = false;
                    a.navigation = None;
                }
            },
        );
        if close {
            self.closing = false;
            self.navigation = None;
        }
    }
    fn point_form(&mut self, ctx: &egui::Context) {
        if self.point_frequency_only {
            self.modal(
                ctx,
                "mouse_point",
                "Datapoint",
                vec2(176., 131.),
                false,
                |a, ui| {
                    group(ui, 4., 9., 158., 27., "Frequency");
                    let scale = [1., 1e3, 1e6, 1e9][a.classic.point_unit];
                    let mut f = a.point_values[2] * 1e6 / scale;
                    if num(
                        ui,
                        &mut a.classic,
                        "point_frequency",
                        6.,
                        20.,
                        70.,
                        &mut f,
                        true,
                    ) {
                        a.point_values[2] = f * scale / 1e6;
                    }
                    combo(
                        ui,
                        "mouse_frequency_unit",
                        80.,
                        20.,
                        77.,
                        &mut a.classic.point_unit,
                        &["Hz", "kHz", "MHz", "GHz"],
                    );
                    let ok = action(ui, 5., 44., 50., "OK", 0);
                    let cancel = action(ui, 59., 44., 50., "Cancel", 1);
                    if action(ui, 113., 44., 50., "Help", 2) {
                        a.classic.help_topic = 5;
                        a.help = true;
                    }
                    if cancel {
                        a.point_dialog = false;
                    }
                    if ok || ui.input(|i| i.key_pressed(Key::Enter)) {
                        a.confirm_point();
                    }
                },
            );
            return;
        }
        self.modal(
            ctx,
            "point",
            "Data Point",
            vec2(204., 286.),
            false,
            |a, ui| {
                group(ui, 3., 7., 187., 64., "");
                for (i, text) in [
                    "impedance (Ω)",
                    "admittance (Siemens)",
                    "reflection coefficient",
                ]
                .iter()
                .enumerate()
                {
                    if radio(ui, 7., 10. + i as f32 * 16., 178., a.point_mode == i, text) {
                        a.point_mode = i;
                    }
                }
                group(
                    ui,
                    4.,
                    80.,
                    77.,
                    26.,
                    if a.point_polar { "Magnitude" } else { "Real" },
                );
                group(
                    ui,
                    110.,
                    80.,
                    76.,
                    26.,
                    if a.point_polar {
                        "Angle [°]"
                    } else {
                        "Imaginary"
                    },
                );
                num(
                    ui,
                    &mut a.classic,
                    "point_re",
                    9.,
                    89.,
                    66.,
                    &mut a.point_values[0],
                    true,
                );
                num(
                    ui,
                    &mut a.classic,
                    "point_im",
                    115.,
                    89.,
                    66.,
                    &mut a.point_values[1],
                    true,
                );
                group(ui, 3., 115., 187., 41., "");
                if radio(ui, 7., 121., 170., !a.point_polar, "cartesian") && a.point_polar {
                    let z = C::from_polar(a.point_values[0], a.point_values[1].to_radians());
                    a.point_values[0] = z.re;
                    a.point_values[1] = z.im;
                    a.point_polar = false;
                }
                if radio(ui, 7., 137., 170., a.point_polar, "polar") && !a.point_polar {
                    let z = c(a.point_values[0], a.point_values[1]);
                    a.point_values[0] = z.norm();
                    a.point_values[1] = z.arg().to_degrees();
                    a.point_polar = true;
                }
                group(ui, 3., 164., 187., 31., "frequency");
                let scale = [1., 1e3, 1e6, 1e9][a.classic.point_unit];
                let mut f = a.point_values[2] * 1e6 / scale;
                if num(
                    ui,
                    &mut a.classic,
                    "point_frequency",
                    8.,
                    173.,
                    100.,
                    &mut f,
                    true,
                ) {
                    a.point_values[2] = f * scale / 1e6;
                }
                combo(
                    ui,
                    "point_frequency_unit",
                    114.,
                    173.,
                    69.,
                    &mut a.classic.point_unit,
                    &["Hz", "kHz", "MHz", "GHz"],
                );
                label(ui, 9., 201., 185., "(exponential format, e.g. 567.34e-3)");
                let ok = action(ui, 35., 221., 50., "OK", 0);
                let cancel = action(ui, 88., 221., 50., "Cancel", 1);
                let help = action(ui, 141., 221., 50., "Help", 2);
                if help {
                    a.classic.help_topic = 5;
                    a.help = true;
                }
                if cancel {
                    a.point_dialog = false;
                }
                if ok || ui.input(|i| i.key_pressed(Key::Enter)) {
                    a.confirm_point();
                }
            },
        );
    }
    fn confirm_point(&mut self) {
        let [x, y, f] = self.point_values;
        let value = if self.point_polar {
            C::from_polar(x, y.to_radians())
        } else {
            c(x, y)
        };
        let z = match self.point_mode {
            1 => inv(value),
            2 => impedance(value, self.p.z0),
            _ => value,
        };
        if f <= 0.
            || !f.is_finite()
            || !z.re.is_finite()
            || !z.im.is_finite()
            || (!self.point_frequency_only
                && ["point_re", "point_im"]
                    .iter()
                    .any(|id| self.classic.blank_fields.contains(*id)))
        {
            self.status = "Invalid datapoint or frequency".into();
            return;
        }
        self.checkpoint();
        if let Some(i) = self.point_edit {
            self.p.points[i].z = z;
            self.p.points[i].hz = f * 1e6;
        } else {
            let i = self.p.points.len();
            self.p.points.push(Datapoint {
                name: format!("DP{}", i + 1),
                z,
                hz: f * 1e6,
            });
            if i == 0 {
                self.p.start = 0;
            }
        }
        self.p.default_hz = f * 1e6;
        self.point_dialog = false;
        self.status.clear();
    }
    fn apply_element(&mut self, close: bool) {
        let mut probe = self.p.clone();
        if let Some(i) = self.element_edit {
            probe.elements[i] = self.draft.clone();
        } else {
            probe.elements.push(self.draft.clone());
        }
        if let Err(e) = probe.validate() {
            self.status = e;
            return;
        }
        if self.classic.element_backup.is_none() {
            self.classic.element_backup = Some(self.p.clone());
            self.checkpoint();
        }
        if let Some(i) = self.element_edit {
            self.p.elements[i] = self.draft.clone();
        } else {
            self.p.elements.push(self.draft.clone());
            self.element_edit = Some(self.p.elements.len() - 1);
        }
        if close {
            self.element_dialog = false;
            self.classic.element_backup = None;
        }
        self.status.clear();
    }
    fn element_form(&mut self, ctx: &egui::Context) {
        let line = matches!(
            self.draft.kind,
            Kind::Line | Kind::OpenStub | Kind::ShortStub
        );
        let initial = line && self.element_edit.is_none();
        let size = if initial {
            vec2(336., 251.)
        } else {
            vec2(250., 450.)
        };
        let close = self.modal(
            ctx,
            "element",
            if initial {
                "Line impedance"
            } else {
                "Edit Element"
            },
            size,
            true,
            |a, ui| {
                if initial {
                    group(ui, 5., 8., 197., 40., "Z0 Line impedance");
                    num(
                        ui,
                        &mut a.classic,
                        "line_z0",
                        11.,
                        16.,
                        155.,
                        &mut a.draft.z0,
                        true,
                    );
                    label(ui, 173., 16., 20., "Ω");
                    group(ui, 210., 8., 108., 40., "Er");
                    num(
                        ui,
                        &mut a.classic,
                        "line_er",
                        218.,
                        16.,
                        90.,
                        &mut a.draft.er,
                        true,
                    );
                    group(ui, 5., 65., 95., 45., "L electr. in λ");
                    num(
                        ui,
                        &mut a.classic,
                        "line_lambda",
                        11.,
                        76.,
                        78.,
                        &mut a.draft.value,
                        a.draft.loss > 0.,
                    );
                    group(ui, 210., 109., 108., 45., "α dB/m (phys. length)");
                    num(
                        ui,
                        &mut a.classic,
                        "line_loss",
                        218.,
                        122.,
                        90.,
                        &mut a.draft.loss,
                        a.draft.kind == Kind::Line,
                    );
                    let (ok, cancel, help) = footer(ui, 171., 318.);
                    if help {
                        a.classic.help_topic = 8;
                        a.help = true;
                    }
                    if cancel {
                        a.element_dialog = false;
                    }
                    if ok {
                        if a.draft.loss > 0. {
                            a.apply_element(true);
                        } else {
                            a.pending = Some(a.draft.clone());
                            a.element_dialog = false;
                        }
                    }
                    return;
                }
                group(ui, 9., 8., 222., 340., "Type in the new value(s)");
                let rlc = matches!(a.draft.kind, Kind::SeriesParallelRlc | Kind::ShuntSeriesRlc);
                let on_r = rlc || matches!(a.draft.kind, Kind::SeriesR | Kind::ShuntR);
                let on_l = rlc || matches!(a.draft.kind, Kind::SeriesL | Kind::ShuntL);
                let on_c = rlc || matches!(a.draft.kind, Kind::SeriesC | Kind::ShuntC);
                for (i, (name, on)) in [("R", on_r), ("L", on_l), ("C", on_c)].iter().enumerate() {
                    let y = 24. + i as f32 * 43.;
                    group(ui, 15., y, 207., 37., name);
                    let scale = match i {
                        0 => [0.001, 1., 1e3, 1e6, 1e9][a.classic.r_unit],
                        1 => [0.001, 1., 1000.][a.classic.l_unit],
                        _ => [1., 1000., 1e6][a.classic.c_unit],
                    };
                    let mut value = match i {
                        0 if on_r => a.draft.value,
                        1 if rlc => a.draft.l_nh,
                        2 if rlc => a.draft.c_pf,
                        _ => a.draft.value,
                    } / scale;
                    if num(
                        ui,
                        &mut a.classic,
                        &format!("component_{i}"),
                        21.,
                        y + 10.,
                        77.,
                        &mut value,
                        *on,
                    ) {
                        let value = value * scale;
                        match i {
                            1 if rlc => a.draft.l_nh = value,
                            2 if rlc => a.draft.c_pf = value,
                            _ => a.draft.value = value,
                        }
                    }
                    let id = format!("component_unit_{i}");
                    match i {
                        0 => {
                            combo(
                                ui,
                                &id,
                                105.,
                                y + 10.,
                                109.,
                                &mut a.classic.r_unit,
                                &["mΩ", "Ω", "kΩ", "MΩ", "GΩ"],
                            );
                        }
                        1 => {
                            combo(
                                ui,
                                &id,
                                105.,
                                y + 10.,
                                109.,
                                &mut a.classic.l_unit,
                                &["pH", "nH", "uH"],
                            );
                        }
                        _ => {
                            combo(
                                ui,
                                &id,
                                105.,
                                y + 10.,
                                109.,
                                &mut a.classic.c_unit,
                                &["pF", "nF", "uF"],
                            );
                        }
                    }
                }
                group(ui, 15., 161., 118., 38., "Trafo n=");
                num(
                    ui,
                    &mut a.classic,
                    "trafo_ratio",
                    21.,
                    171.,
                    103.,
                    &mut a.draft.value,
                    a.draft.kind == Kind::Transformer,
                );
                group(ui, 15., 229., 137., 40., "Z0 Line impedance");
                num(
                    ui,
                    &mut a.classic,
                    "edit_line_z0",
                    21.,
                    241.,
                    103.,
                    &mut a.draft.z0,
                    line,
                );
                label(ui, 128., 241., 20., "Ω");
                group(ui, 158., 229., 64., 40., "Er");
                num(
                    ui,
                    &mut a.classic,
                    "edit_line_er",
                    165.,
                    241.,
                    50.,
                    &mut a.draft.er,
                    line,
                );
                let mut lambda = a.draft.value;
                let mut electrical = a.draft.value * LIGHT / a.draft.reference_hz * 1000.;
                let mut physical = a.draft.physical_m() * 1000.;
                for (i, name) in ["L electr. in λ", "L electr. in mm", "L phys. in mm"]
                    .iter()
                    .enumerate()
                {
                    group(ui, 15. + i as f32 * 70., 280., 66., 39., name);
                }
                if num(
                    ui,
                    &mut a.classic,
                    "edit_lambda",
                    20.,
                    292.,
                    55.,
                    &mut lambda,
                    line,
                ) {
                    a.draft.value = lambda;
                }
                if num(
                    ui,
                    &mut a.classic,
                    "edit_electrical",
                    90.,
                    292.,
                    55.,
                    &mut electrical,
                    line,
                ) {
                    a.draft.value = electrical / 1000. * a.draft.reference_hz / LIGHT;
                }
                if num(
                    ui,
                    &mut a.classic,
                    "edit_physical",
                    160.,
                    292.,
                    55.,
                    &mut physical,
                    line,
                ) {
                    a.draft.value =
                        physical / 1000. * a.draft.reference_hz * a.draft.er.sqrt() / LIGHT;
                }
                group(ui, 140., 327., 82., 35., "α dB/m (phys. length)");
                num(
                    ui,
                    &mut a.classic,
                    "edit_loss",
                    148.,
                    337.,
                    65.,
                    &mut a.draft.loss,
                    a.draft.kind == Kind::Line,
                );
                if green_button(ui, 9., 362., 222., 18., "Draw") {
                    a.apply_element(false);
                }
                let (ok, cancel, help) = footer(ui, 385., 231.);
                if help {
                    a.classic.help_topic = 8;
                    a.help = true;
                }
                if ok {
                    a.apply_element(true);
                }
                if cancel {
                    if let Some(p) = a.classic.element_backup.take() {
                        a.p = p;
                    }
                    a.element_dialog = false;
                }
            },
        );
        if close {
            if let Some(p) = self.classic.element_backup.take() {
                self.p = p;
            }
            self.element_dialog = false;
        }
    }
    fn sweep_form(&mut self, ctx: &egui::Context) {
        if self.classic.sweep_backup.is_none() {
            self.classic.sweep_backup = Some(self.p.sweep.clone());
        }
        let close = self.modal(ctx, "sweep", "Sweep", vec2(362., 347.), true, |a, ui| {
            group(ui, 4., 10., 187., 199., "");
            group(ui, 196., 10., 156., 199., "");
            if radio(
                ui,
                10.,
                15.,
                178.,
                a.p.sweep.datapoints,
                "sweep over datapoints",
            ) {
                a.p.sweep.datapoints = true;
            }
            label(ui, 23., 45., 168., "Sweep over all datapoints");
            label(
                ui,
                23.,
                59.,
                168.,
                "The datapoints may have different frequencies.",
            );
            if radio(
                ui,
                198.,
                15.,
                154.,
                !a.p.sweep.datapoints,
                "sweep over frequency (single datapoint)",
            ) {
                a.p.sweep.datapoints = false;
            }
            let mut step =
                a.p.sweep
                    .step_hz
                    .unwrap_or((a.p.sweep.stop - a.p.sweep.start) / (a.p.sweep.count - 1) as f64);
            let mut changed = false;
            for (i, name) in ["start frequency", "stop frequency", "step"]
                .iter()
                .enumerate()
            {
                let y = 41. + i as f32 * 40.;
                group(ui, 209., y, 135., 32., name);
                let scale = [1., 1e3, 1e6, 1e9][a.classic.sweep_units[i]];
                let mut value = match i {
                    0 => a.p.sweep.start,
                    1 => a.p.sweep.stop,
                    _ => step,
                } / scale;
                if num(
                    ui,
                    &mut a.classic,
                    &format!("sweep_{i}"),
                    213.,
                    y + 8.,
                    68.,
                    &mut value,
                    !a.p.sweep.datapoints,
                ) {
                    changed = true;
                    match i {
                        0 => a.p.sweep.start = value * scale,
                        1 => a.p.sweep.stop = value * scale,
                        _ => step = value * scale,
                    }
                }
                enabled_at(ui, !a.p.sweep.datapoints, |ui| {
                    combo(
                        ui,
                        &format!("sweep_unit_{i}"),
                        284.,
                        y + 8.,
                        47.,
                        &mut a.classic.sweep_units[i],
                        &["Hz", "kHz", "MHz", "GHz"],
                    );
                });
            }
            if changed && step > 0. && a.p.sweep.stop > a.p.sweep.start {
                a.p.sweep.count = (((a.p.sweep.stop - a.p.sweep.start) / step).floor() as usize
                    + 1)
                .clamp(2, 10001);
                a.p.sweep.step_hz = Some(step);
            }
            group(ui, 209., 161., 135., 32., "number of points");
            field(ui, 213., 169., 68., &a.p.sweep.count.to_string());
            let draw = green_button(ui, 196., 213., 156., 18., "Draw");
            let ok = action(ui, 196., 235., 50., "OK", 0);
            let cancel = action(ui, 250., 235., 50., "Cancel", 1);
            let help = action(ui, 304., 235., 50., "Help", 2);
            if help {
                a.classic.help_topic = 22;
                a.help = true;
            }
            if draw || ok {
                match a.p.validate() {
                    Ok(()) => {
                        let mut before = a.p.clone();
                        if let Some(original) = &a.classic.sweep_backup {
                            before.sweep = original.clone();
                        }
                        a.history.record(&before);
                        a.p.sweep.visible = true;
                        if ok {
                            a.sweep_dialog = false;
                            a.classic.sweep_backup = None;
                        }
                    }
                    Err(e) => a.status = e,
                }
            }
            if cancel {
                if let Some(s) = a.classic.sweep_backup.take() {
                    a.p.sweep = s;
                }
                a.sweep_dialog = false;
            }
        });
        if close {
            if let Some(s) = self.classic.sweep_backup.take() {
                self.p.sweep = s;
            }
            self.sweep_dialog = false;
        }
    }
    fn settings_form(&mut self, ctx: &egui::Context) {
        if self.classic.style_backup.is_none() {
            let mut backup = self.classic.clone();
            backup.style_backup = None;
            self.classic.style_backup = Some(Box::new(backup));
            self.classic.style_project = Some(self.p.clone());
        }
        let close = self.modal(
            ctx,
            "settings",
            "Settings",
            vec2(524., 433.),
            true,
            |a, ui| {
                tabs(
                    ui,
                    &mut a.classic.settings_tab,
                    &["Smith-Chart", "S-Plot", "General"],
                );
                let tab = a.classic.settings_tab;
                if tab == 0 {
                    group(ui, 15., 42., 341., 213., "Lines and colors");
                    label(ui, 110., 48., 100., "Color line width");
                    label(ui, 276., 48., 75., "Color Line width");
                    let left = [
                        "Z-Plane lines",
                        "Y-Plane lines",
                        "Datapoints circles",
                        "Q-Circles",
                        "Stability-circles",
                        "Gain-circles",
                        "VSWR-circles",
                        "Noise-circles",
                        "Sweep-lines",
                        "|Γ|=1-circle",
                    ];
                    for (i, name) in left.iter().enumerate() {
                        a.color_setting(ui, name, i, 22., 65. + i as f32 * 19., 106., 128.);
                    }
                    for (i, (name, row)) in [
                        ("Ref. Z-circle lines", 10),
                        ("Ref. Y-circle lines", 11),
                        ("Datapoints lines", 12),
                        ("Temp. lines", 13),
                        ("Element lines", 14),
                        ("Sweeppoint circles", 15),
                    ]
                    .iter()
                    .enumerate()
                    {
                        let y = 65. + if i == 5 { 8. } else { i as f32 } * 19.;
                        a.color_setting(ui, name, *row, 181., y, 274., 296.);
                    }
                    group(ui, 363., 42., 136., 183., "Smith Chart");
                    for (i, text) in [
                        "Show own labels",
                        "Z-plane (on/off)",
                        "Y-plane (on/off)",
                        "Connect datapoints",
                        "Chartlabels (on/off)",
                        "Label DP (on/off)",
                        "Label TP (on/off)",
                        "Label circles (on/off)",
                        "Label SP (on/off)",
                        "Expanded",
                    ]
                    .iter()
                    .enumerate()
                    {
                        enabled_at(ui, i != 0, |ui| {
                            checkbox(
                                ui,
                                371.,
                                53. + i as f32 * 16.,
                                125.,
                                &mut a.classic.flags[i],
                                text,
                            );
                        });
                    }
                    a.p.settings.z_grid = a.classic.flags[1];
                    a.p.settings.y_grid = a.classic.flags[2];
                    a.p.settings.labels = a.classic.flags[4];
                    a.p.settings.expanded = a.classic.flags[9];
                    group(ui, 15., 262., 224., 30., "General");
                    label(ui, 23., 270., 87., "Default Zo");
                    let z_scale = [1e9, 1e6, 1e3, 1.][a.classic.settings_z0_unit];
                    let mut z0 = a.p.z0 / z_scale;
                    if num(
                        ui,
                        &mut a.classic,
                        "settings_z0",
                        113.,
                        271.,
                        44.,
                        &mut z0,
                        true,
                    ) && z0 > 0.
                    {
                        a.p.z0 = z0 * z_scale;
                    }
                    combo(
                        ui,
                        "settings_z0_unit",
                        160.,
                        271.,
                        39.,
                        &mut a.classic.settings_z0_unit,
                        &["GΩ", "MΩ", "kΩ", "Ω"],
                    );
                    group(ui, 15., 300., 224., 33., "Datapoint");
                    label(ui, 23., 309., 89., "Default frequency");
                    let f_scale = [1e9, 1e6, 1e3, 1.][a.classic.settings_frequency_unit];
                    let mut f = a.p.default_hz / f_scale;
                    if num(
                        ui,
                        &mut a.classic,
                        "settings_f",
                        113.,
                        310.,
                        44.,
                        &mut f,
                        true,
                    ) && f > 0.
                    {
                        a.p.default_hz = f * f_scale;
                    }
                    combo(
                        ui,
                        "settings_frequency_unit",
                        160.,
                        310.,
                        39.,
                        &mut a.classic.settings_frequency_unit,
                        &["GHz", "MHz", "kHz", "Hz"],
                    );
                    group(ui, 245., 285., 111., 48., "Circuit orientation");
                    radio(ui, 252., 294., 100., true, "Toward generator");
                    group(ui, 363., 225., 136., 108., "Schematic");
                    checkbox(
                        ui,
                        371.,
                        243.,
                        125.,
                        &mut a.classic.flags[10],
                        "Show auto labels",
                    );
                    enabled_at(ui, false, |ui| {
                        checkbox(
                            ui,
                            371.,
                            259.,
                            125.,
                            &mut a.classic.flags[11],
                            "Show own labels",
                        );
                    });
                    label(ui, 371., 277., 125., "Schematic orientation");
                    if radio(ui, 371., 295., 120., a.p.settings.vertical, "Vertical") {
                        a.p.settings.vertical = true;
                    }
                    if radio(ui, 371., 309., 120., !a.p.settings.vertical, "Horizontal") {
                        a.p.settings.vertical = false;
                    }
                }
                if tab == 1 {
                    group(ui, 15., 42., 250., 190., "Lines and colors");
                    for (i, (name, row)) in [
                        ("Polar grid lines", 16),
                        ("MAG line", 17),
                        ("MSG line", 18),
                        ("K line", 19),
                        ("μ line", 20),
                        ("Return Loss S11 line", 21),
                        ("Return Loss S22 line", 22),
                        ("Grid lines Gain Graph", 23),
                        ("Grid lines Return Loss Graph", 24),
                    ]
                    .iter()
                    .enumerate()
                    {
                        a.color_setting(ui, name, *row, 23., 56. + i as f32 * 19., 181., 203.);
                    }
                    group(ui, 277., 42., 222., 190., "S11 to S22 Window");
                    for (i, text) in [
                        "Z-plane (on/off)",
                        "Y-plane (on/off)",
                        "Chartlabels (on/off)",
                        "Datapoint circles (on/off)",
                    ]
                    .iter()
                    .enumerate()
                    {
                        checkbox(
                            ui,
                            284.,
                            54. + i as f32 * 20.,
                            208.,
                            &mut a.classic.flags[12 + i],
                            text,
                        );
                    }
                }
                if tab == 2 {
                    group(ui, 15., 42., 154., 40., "Default start mode");
                    if radio(ui, 22., 51., 78., !a.classic.default_splot, "Smith Chart") {
                        a.classic.default_splot = false;
                    }
                    if radio(ui, 106., 51., 60., a.classic.default_splot, "S-Plot") {
                        a.classic.default_splot = true;
                    }
                }
                ui.painter().line_segment(
                    [
                        at(ui, 0., 338., 509., 0.).left_top(),
                        at(ui, 0., 338., 509., 0.).right_top(),
                    ],
                    Stroke::new(1.0_f32, Color32::from_gray(150)),
                );
                if button(ui, 241., 345., 91., 20., "Restore defaults") {
                    let defaults = ClassicState::default();
                    a.classic.color_rows = defaults.color_rows;
                    a.classic.widths = defaults.widths;
                    a.classic.flags = defaults.flags;
                    a.classic.default_splot = defaults.default_splot;
                    a.classic.settings_z0_unit = defaults.settings_z0_unit;
                    a.classic.settings_frequency_unit = defaults.settings_frequency_unit;
                    a.p.z0 = 50.;
                    a.p.default_hz = 5e8;
                    a.p.settings = Settings::default();
                }
                let (ok, cancel, help) = footer(ui, 345., 509.);
                if help {
                    a.classic.help_topic = 29;
                    a.help = true;
                }
                if ok {
                    a.classic.style_backup = None;
                    a.classic.style_project = None;
                    a.settings = false;
                }
                if cancel {
                    a.cancel_settings();
                }
            },
        );
        if close {
            self.cancel_settings();
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn color_setting(
        &mut self,
        ui: &mut Ui,
        name: &str,
        index: usize,
        x: f32,
        y: f32,
        cx: f32,
        wx: f32,
    ) {
        label(ui, x, y, cx - x - 3., name);
        let color = self.classic.color_rows[index];
        let rect = at(ui, cx, y + 1., 19., 14.);
        let response = ui.interact(
            rect,
            ui.make_persistent_id(("color", index)),
            Sense::click(),
        );
        ui.painter()
            .rect_filled(rect, 0., Color32::from_rgb(color[0], color[1], color[2]));
        ui.painter().rect_stroke(
            rect,
            0.,
            Stroke::new(0.5f32, Color32::GRAY),
            StrokeKind::Inside,
        );
        if response.clicked()
            && let Some(color) = native::choose_color(color, &mut self.classic.custom_colors)
        {
            self.classic.color_rows[index] = color;
        }
        let mut choice = (self.classic.widths[index].round() as usize)
            .saturating_sub(1)
            .min(4);
        if combo(
            ui,
            &format!("width_{index}"),
            wx,
            y,
            39.,
            &mut choice,
            &["1", "2", "3", "4", "5"],
        ) {
            self.classic.widths[index] = (choice + 1) as f32;
        }
    }
    fn cancel_settings(&mut self) {
        if let Some(p) = self.classic.style_project.take() {
            self.p = p;
        }
        if let Some(backup) = self.classic.style_backup.take() {
            self.classic.color_rows = backup.color_rows;
            self.classic.widths = backup.widths;
            self.classic.flags = backup.flags;
            self.classic.default_splot = backup.default_splot;
            self.classic.settings_z0_unit = backup.settings_z0_unit;
            self.classic.settings_frequency_unit = backup.settings_frequency_unit;
        }
        self.settings = false;
    }
    fn tune_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(ctx, "tune", "Tune", vec2(493., 281.), true, |a, ui| {
            if a.tune_centers.len() != a.p.elements.len() {
                a.tune_centers = a.p.elements.iter().map(|e| e.value).collect();
                a.tune_z0_centers = a.p.elements.iter().map(|e| e.z0).collect();
                a.tune_lc_centers = a.p.elements.iter().map(|e| [e.l_nh, e.c_pf]).collect();
            }
            a.classic.tune_selected.retain(|i| *i < a.p.elements.len());
            let mut remove = None;
            let before = a.p.clone();
            egui::ScrollArea::horizontal().show(ui, |ui| {
                let width = a
                    .classic
                    .tune_selected
                    .iter()
                    .map(|i| {
                        if matches!(
                            a.p.elements[*i].kind,
                            Kind::Line | Kind::OpenStub | Kind::ShortStub
                        ) {
                            160.
                        } else if matches!(
                            a.p.elements[*i].kind,
                            Kind::SeriesParallelRlc | Kind::ShuntSeriesRlc
                        ) {
                            244.
                        } else {
                            82.
                        }
                    })
                    .sum::<f32>()
                    .max(455.);
                ui.set_min_size(vec2(width, 185.));
                let mut x = 0.;
                for i in a.classic.tune_selected.clone() {
                    let line = matches!(
                        a.p.elements[i].kind,
                        Kind::Line | Kind::OpenStub | Kind::ShortStub
                    );
                    let rlc = matches!(
                        a.p.elements[i].kind,
                        Kind::SeriesParallelRlc | Kind::ShuntSeriesRlc
                    );
                    if rlc {
                        group(ui, x + 2., 4., 240., 179., "");
                        crate::schematic::toolbar_symbol(
                            ui.painter(),
                            at(ui, x + 9., 9., 25., 24.),
                            &a.p.elements[i],
                        );
                        if button(ui, x + 41., 13., 35., 17., "Remove") {
                            remove = Some(i);
                        }
                        let e = &mut a.p.elements[i];
                        tune_slider(ui, x, &mut e.value, a.tune_centers[i], "R [Ω]");
                        tune_slider(ui, x + 80., &mut e.l_nh, a.tune_lc_centers[i][0], "L [nH]");
                        tune_slider(ui, x + 160., &mut e.c_pf, a.tune_lc_centers[i][1], "C [pF]");
                        x += 244.;
                        continue;
                    }
                    group(ui, x + 2., 4., if line { 156. } else { 78. }, 179., "");
                    crate::schematic::toolbar_symbol(
                        ui.painter(),
                        at(ui, x + 9., 9., 25., 24.),
                        &a.p.elements[i],
                    );
                    if button(ui, x + 41., 13., 35., 17., "Remove") {
                        remove = Some(i);
                    }
                    let base = a.tune_centers[i];
                    let e = &mut a.p.elements[i];
                    label(ui, x + 9., 42., 70., e.kind.unit());
                    ui.scope_builder(
                        UiBuilder::new().max_rect(at(ui, x + 10., 60., 19., 111.)),
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut e.value, base * 0.8..=base * 1.2)
                                    .vertical()
                                    .show_value(false),
                            );
                        },
                    );
                    label(ui, x + 37., 60., 45., "Max.");
                    field(ui, x + 37., 75., 35., &format!("{:.2}", base * 1.2));
                    label(ui, x + 37., 101., 40., format!("Value\n{:.2}", e.value));
                    label(ui, x + 37., 141., 45., "Min.");
                    field(ui, x + 37., 157., 35., &format!("{:.2}", base * 0.8));
                    if line {
                        let z0 = a.tune_z0_centers[i];
                        label(ui, x + 87., 42., 70., "Z0 Line impedance");
                        ui.scope_builder(
                            UiBuilder::new().max_rect(at(ui, x + 90., 60., 19., 111.)),
                            |ui| {
                                ui.add(
                                    egui::Slider::new(&mut e.z0, z0 * 0.8..=z0 * 1.2)
                                        .vertical()
                                        .show_value(false),
                                );
                            },
                        );
                        field(ui, x + 116., 75., 35., &format!("{:.2}", z0 * 1.2));
                        field(ui, x + 116., 111., 35., &format!("{:.2}", e.z0));
                        field(ui, x + 116., 157., 35., &format!("{:.2}", z0 * 0.8));
                    }
                    x += if line { 160. } else { 82. };
                }
            });
            if let Some(i) = remove {
                a.classic.tune_selected.retain(|n| *n != i);
            }
            if a.p != before {
                a.history.record(&before);
            }
            if button(ui, 8., 198., 110., 19., "Reset Min/Max") {
                a.tune_centers = a.p.elements.iter().map(|e| e.value).collect();
                a.tune_z0_centers = a.p.elements.iter().map(|e| e.z0).collect();
                a.tune_lc_centers = a.p.elements.iter().map(|e| [e.l_nh, e.c_pf]).collect();
            }
            if button(ui, 124., 198., 110., 19., "Remove all") {
                a.classic.tune_selected.clear();
            }
            if action(ui, 8., 221., 52., "OK", 0) {
                a.tune = false;
            }
            if action(ui, 66., 221., 52., "Help", 2) {
                a.classic.help_topic = 9;
                a.help = true;
            }
            label(
                ui,
                211.,
                220.,
                260.,
                "Select element to tune in schematic window",
            );
        });
        if close {
            self.tune = false;
        }
    }
    fn circles_form(&mut self, ctx: &egui::Context) {
        if self.classic.circle_backup.is_none() {
            self.classic.circle_backup = Some(self.p.circles.clone());
        }
        let close = self.modal(
            ctx,
            "circles",
            "Circles",
            vec2(
                426.,
                if self.classic.circle_tab == 4 {
                    473.
                } else {
                    435.
                },
            ),
            false,
            |a, ui| {
                tabs(
                    ui,
                    &mut a.classic.circle_tab,
                    &["Q", "Gain", "VSWR", "Stability", "Noise Figure"],
                );
                let tab = a.classic.circle_tab;
                if tab == 0 || tab == 2 {
                    let q = tab == 0;
                    let values = if q {
                        [10., 5., 2., 1., 0.5, 0.2]
                    } else {
                        [10., 5., 3., 2., 1.5, 1.2]
                    };
                    group(ui, 17., 43., 133., 128., "Defined");
                    for (i, v) in values.iter().enumerate() {
                        let mut checked = a.p.circles.iter().any(|c| {
                            if q {
                                c.q == Some(*v)
                            } else {
                                c.label == format!("VSWR = {v:.3}")
                            }
                        });
                        if checkbox(
                            ui,
                            21.,
                            50. + i as f32 * 16.,
                            59.,
                            &mut checked,
                            &format!("{v:.2}"),
                        ) {
                            a.toggle_contour(q, *v, checked);
                        }
                    }
                    if button(ui, 84., 82., 59., 19., "Select all") {
                        for v in values {
                            a.toggle_contour(q, v, true);
                        }
                    }
                    if button(ui, 84., 106., 59., 19., "Clear all") {
                        for v in values {
                            a.toggle_contour(q, v, false);
                        }
                    }
                    group(ui, 165., 43., 176., 128., "Select other");
                    let text = if q {
                        &mut a.classic.q_other
                    } else {
                        &mut a.classic.vswr_other
                    };
                    ui.put(
                        at(ui, 171., 51., 83., 18.),
                        egui::TextEdit::singleline(text).font(FontId::proportional(10.)),
                    );
                    let mut add = false;
                    if button(ui, 279., 68., 54., 19., "Insert") {
                        add = true;
                    }
                    if add {
                        let text = if q {
                            &a.classic.q_other
                        } else {
                            &a.classic.vswr_other
                        };
                        if let Ok(v) = text.parse::<f64>()
                            && v.is_finite()
                            && v >= if q { 0. } else { 1. }
                        {
                            a.toggle_contour(q, v, true);
                        }
                    }
                    let candidates: Vec<_> =
                        a.p.circles
                            .iter()
                            .enumerate()
                            .filter(|c| {
                                if q {
                                    c.1.q.is_some()
                                } else {
                                    c.1.label.starts_with("VSWR")
                                }
                            })
                            .map(|(i, c)| (i, c.label.clone()))
                            .collect();
                    ui.scope_builder(
                        UiBuilder::new().max_rect(at(ui, 171., 72., 83., 91.)),
                        |ui| {
                            ui.painter().rect_filled(ui.max_rect(), 0., Color32::WHITE);
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, label) in candidates {
                                    if ui
                                        .selectable_label(
                                            a.classic.selected_circle == Some(i),
                                            label,
                                        )
                                        .clicked()
                                    {
                                        a.classic.selected_circle = Some(i);
                                    }
                                }
                            });
                        },
                    );
                    if button(ui, 279., 92., 54., 19., "Delete")
                        && let Some(i) = a.classic.selected_circle.take()
                    {
                        a.p.circles.remove(i);
                    }
                    if button(ui, 279., 116., 54., 19., "Clear all") {
                        a.p.circles.retain(|c| {
                            if q {
                                c.q.is_none()
                            } else {
                                !c.label.starts_with("VSWR")
                            }
                        });
                    }
                } else if tab == 1 || tab == 3 {
                    group(
                        ui,
                        6.,
                        28.,
                        188.,
                        136.,
                        if tab == 1 {
                            "Select gain"
                        } else {
                            "Stability circle"
                        },
                    );
                    if tab == 1 {
                        if radio(
                            ui,
                            12.,
                            39.,
                            106.,
                            a.circle_kind == 3,
                            "Available Power Gain",
                        ) {
                            a.circle_kind = 3;
                            a.circle_source = true;
                        }
                        if radio(
                            ui,
                            115.,
                            39.,
                            80.,
                            a.circle_kind == 4,
                            "Operating Power Gain",
                        ) {
                            a.circle_kind = 4;
                            a.circle_source = false;
                        }
                        if radio(ui, 12., 67., 177., a.circle_kind == 5, "Unilateral") {
                            a.circle_kind = 5;
                        }
                    } else {
                        a.circle_kind = 2;
                    }
                    if green_button(
                        ui,
                        12.,
                        87.,
                        83.,
                        23.,
                        if tab == 1 { "Source" } else { "Input" },
                    ) {
                        a.circle_source = true;
                        if a.circle_kind == 4 {
                            a.circle_kind = 3;
                        }
                    }
                    if button(
                        ui,
                        102.,
                        87.,
                        83.,
                        23.,
                        if tab == 1 { "Load" } else { "Output" },
                    ) {
                        a.circle_source = false;
                        if a.circle_kind == 3 {
                            a.circle_kind = 4;
                        }
                    }
                    group(ui, 207., 28., 188., 47., "Import");
                    if button(ui, 214., 37., 105., 23., "S-Parameter") {
                        a.import_for_circles();
                    }
                    group(
                        ui,
                        207.,
                        88.,
                        188.,
                        76.,
                        if tab == 1 {
                            "Gain to show"
                        } else {
                            "Stability"
                        },
                    );
                    let st = SPoint {
                        hz: 1.,
                        s: a.circle_s,
                    }
                    .stability();
                    label(ui, 214., 98., 65., "K");
                    field(ui, 214., 114., 78., &format!("{:.2}", st.k));
                    if tab == 1 {
                        label(ui, 300., 98., 88., "Power gain [dB]");
                        num(
                            ui,
                            &mut a.classic,
                            "gain_db",
                            300.,
                            114.,
                            85.,
                            &mut a.circle_value,
                            true,
                        );
                        label(ui, 214., 137., 80., "Gmax [dB]");
                        field(ui, 295., 137., 90., &format!("{:.2}", st.gain_db));
                    } else {
                        label(ui, 300., 98., 88., "μ");
                        field(ui, 300., 114., 85., &format!("{:.2}", st.mu));
                    }
                    group(ui, 6., 174., 88., 42., "Frequency");
                    field(
                        ui,
                        12.,
                        185.,
                        75.,
                        &a.classic
                            .circle_frequency
                            .map_or(String::new(), |f| format!("{:.2}MHz", f / 1e6)),
                    );
                    for (j, x, y) in [
                        (0, 6., 229.),
                        (2, 207., 229.),
                        (1, 6., 308.),
                        (3, 207., 308.),
                    ] {
                        a.s_parameter_group(ui, j, x, y);
                    }
                    if green_button(ui, 207., 357., 188., 23., "Draw") {
                        if tab == 1 && !matches!(a.circle_kind, 3..=5) {
                            a.circle_kind = 3;
                        }
                        a.draw_circle();
                    }
                } else {
                    group(
                        ui,
                        8.,
                        28.,
                        261.,
                        108.,
                        a.p.network.as_ref().map_or("", |n| n.name.as_str()),
                    );
                    let noises =
                        a.p.network
                            .as_ref()
                            .map(|n| n.noise.clone())
                            .unwrap_or_default();
                    ui.scope_builder(
                        UiBuilder::new().max_rect(at(ui, 14., 38., 249., 91.)),
                        |ui| {
                            egui::ScrollArea::both().show(ui, |ui| {
                                egui::Grid::new("noise_table")
                                    .spacing([8., 2.])
                                    .show(ui, |ui| {
                                        for label in ["f [MHz]", "Fmin", "mag", "angle[°]", "rn"] {
                                            ui.label(label);
                                        }
                                        ui.end_row();
                                        for n in noises {
                                            if ui
                                                .selectable_label(
                                                    a.classic.circle_frequency == Some(n.hz),
                                                    format!("{:.2}", n.hz / 1e6),
                                                )
                                                .clicked()
                                            {
                                                a.classic.circle_frequency = Some(n.hz);
                                                a.noise_values = [
                                                    n.fmin_db + 0.1,
                                                    n.fmin_db,
                                                    n.opt.norm(),
                                                    n.opt.arg().to_degrees(),
                                                    n.rn,
                                                ];
                                            }
                                            for value in [
                                                n.fmin_db,
                                                n.opt.norm(),
                                                n.opt.arg().to_degrees(),
                                                n.rn,
                                            ] {
                                                ui.label(format!("{value:.2}"));
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                        },
                    );
                    group(ui, 283., 28., 119., 108., "Import");
                    if button(ui, 289., 42., 108., 25., "S-Parameter") {
                        a.import_for_circles();
                    }
                    group(ui, 8., 151., 104., 47., "Frequency");
                    field(
                        ui,
                        14.,
                        167.,
                        91.,
                        &a.classic
                            .circle_frequency
                            .map_or(String::new(), |f| format!("{:.2}MHz", f / 1e6)),
                    );
                    group(ui, 283., 151., 119., 47., "Noise figure to show [dB]");
                    num(
                        ui,
                        &mut a.classic,
                        "noise_nf",
                        289.,
                        167.,
                        105.,
                        &mut a.noise_values[0],
                        true,
                    );
                    if green_button(ui, 283., 208., 119., 25., "Draw") {
                        a.circle_kind = 6;
                        a.draw_circle();
                    }
                    group(ui, 8., 220., 261., 38., "Minimum noise figure NFmin [dB]");
                    num(
                        ui,
                        &mut a.classic,
                        "noise_min",
                        14.,
                        231.,
                        105.,
                        &mut a.noise_values[1],
                        true,
                    );
                    group(
                        ui,
                        8.,
                        272.,
                        261.,
                        78.,
                        "Reflection coefficient corresponding to NFmin",
                    );
                    let polar = a.classic.noise_polar;
                    label(ui, 15., 279., 90., if polar { "mag" } else { "Real" });
                    label(
                        ui,
                        143.,
                        279.,
                        90.,
                        if polar { "angle [°]" } else { "Imaginary" },
                    );
                    let z = C::from_polar(a.noise_values[2], a.noise_values[3].to_radians());
                    let mut re = if polar { z.norm() } else { z.re };
                    let mut im = if polar { z.arg().to_degrees() } else { z.im };
                    let changed = num(
                        ui,
                        &mut a.classic,
                        "noise_re",
                        14.,
                        296.,
                        110.,
                        &mut re,
                        true,
                    ) | num(
                        ui,
                        &mut a.classic,
                        "noise_im",
                        143.,
                        296.,
                        110.,
                        &mut im,
                        true,
                    );
                    if changed {
                        let z = if polar {
                            C::from_polar(re, im.to_radians())
                        } else {
                            c(re, im)
                        };
                        a.noise_values[2] = z.norm();
                        a.noise_values[3] = z.arg().to_degrees();
                    }
                    if radio(ui, 15., 324., 54., polar, "Polar") {
                        a.classic.noise_polar = true;
                    }
                    if radio(ui, 70., 324., 80., !polar, "Cartesian") {
                        a.classic.noise_polar = false;
                    }
                    group(ui, 8., 362., 261., 37., "Equivalent input noise resistance");
                    let mut rn =
                        a.noise_values[4] * if a.classic.noise_normal { 1. } else { a.p.z0 };
                    if num(
                        ui,
                        &mut a.classic,
                        "noise_rn",
                        14.,
                        373.,
                        80.,
                        &mut rn,
                        true,
                    ) {
                        a.noise_values[4] = rn / if a.classic.noise_normal { 1. } else { a.p.z0 };
                    }
                    if radio(ui, 104., 373., 76., a.classic.noise_normal, "Normalized") {
                        a.classic.noise_normal = true;
                    }
                    if radio(
                        ui,
                        181.,
                        373.,
                        82.,
                        !a.classic.noise_normal,
                        "Not normalized",
                    ) {
                        a.classic.noise_normal = false;
                    }
                }
                let y = if tab == 4 { 421. } else { 383. };
                let (ok, cancel, help) = footer(ui, y, 411.);
                if ok {
                    a.classic.circle_backup = None;
                    a.circle_dialog = false;
                }
                if cancel {
                    if let Some(c) = a.classic.circle_backup.take() {
                        a.p.circles = c;
                    }
                    a.circle_dialog = false;
                }
                if help {
                    a.classic.help_topic = 23;
                    a.help = true;
                }
            },
        );
        if close {
            self.circle_dialog = false;
        }
    }
    fn toggle_contour(&mut self, q: bool, value: f64, on: bool) {
        if on {
            if self.p.circles.iter().any(|c| {
                if q {
                    c.q == Some(value)
                } else {
                    c.label == format!("VSWR = {value:.3}")
                }
            }) {
                return;
            }
            self.circle_kind = if q { 1 } else { 0 };
            self.circle_value = value;
            self.draw_circle();
        } else {
            self.p.circles.retain(|c| {
                if q {
                    c.q != Some(value)
                } else {
                    c.label != format!("VSWR = {value:.3}")
                }
            });
        }
    }
    fn s_parameter_group(&mut self, ui: &mut Ui, j: usize, x: f32, y: f32) {
        let name = ["S11", "S21", "S12", "S22"][j];
        group(ui, x, y, 188., 70., name);
        let format = self.classic.circle_formats[j];
        label(
            ui,
            x + 7.,
            y + 4.,
            80.,
            match format {
                0 => "Magnitude",
                1 => "Real",
                _ => "dB",
            },
        );
        label(
            ui,
            x + 100.,
            y + 4.,
            78.,
            if format == 1 {
                "Imaginary"
            } else {
                "Angle [°]"
            },
        );
        let s = self.circle_s[j];
        let mut first = match format {
            0 => s.norm(),
            1 => s.re,
            _ => db(s.norm()),
        };
        let mut second = if format == 1 {
            s.im
        } else {
            s.arg().to_degrees()
        };
        let changed = num(
            ui,
            &mut self.classic,
            &format!("{name}_a"),
            x + 7.,
            y + 21.,
            78.,
            &mut first,
            true,
        ) | num(
            ui,
            &mut self.classic,
            &format!("{name}_b"),
            x + 100.,
            y + 21.,
            78.,
            &mut second,
            true,
        );
        if changed {
            self.circle_s[j] = if format == 1 {
                c(first, second)
            } else {
                C::from_polar(
                    if format == 2 {
                        10f64.powf(first / 20.)
                    } else {
                        first
                    },
                    second.to_radians(),
                )
            };
            self.classic.circle_frequency = None;
        }
        for (i, label) in ["Polar", "Cartesian", "dB"].iter().enumerate() {
            if radio(
                ui,
                x + 7. + [0., 50., 119.][i],
                y + 47.,
                if i == 1 { 70. } else { 51. },
                format == i,
                label,
            ) {
                self.classic.circle_formats[j] = i;
            }
        }
    }
    fn import_for_circles(&mut self) {
        self.classic.import_for_circle = true;
        self.import_dialog = Some(0);
        if let Some(n) = &self.p.network {
            self.import_selection = vec![false; n.points.len()];
            self.import_dialog = Some(0);
        }
    }
    fn import_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(
            ctx,
            "import",
            "s-Parameters",
            vec2(604., 428.),
            true,
            |a, ui| {
                let mut port = a.import_dialog.unwrap_or(0);
                if !a.classic.import_for_circle {
                    if radio(ui, 4., 0., 70., port == 0, "S11") {
                        port = 0;
                    }
                    if a.p.network.as_ref().is_some_and(|n| n.ports == 2)
                        && radio(ui, 80., 0., 70., port == 1, "S22")
                    {
                        port = 1;
                    }
                }
                a.import_dialog = Some(port);
                let Some(n) = a.p.network.clone() else {
                    let bounds = at(ui, 0., 20., 590., 320.);
                    ui.painter().rect_filled(bounds, 0., DESKTOP);
                    if button(ui, 292., 358., 89., 22., "Select File") {
                        a.open_network_dialog();
                    }
                    let (ok, cancel, help) = footer(ui, 358., 590.);
                    if ok || cancel {
                        a.import_dialog = None;
                        a.classic.import_for_circle = false;
                    }
                    if help {
                        a.classic.help_topic = 39;
                        a.help = true;
                    }
                    return;
                };
                if a.import_selection.len() != n.points.len() {
                    a.import_selection = vec![false; n.points.len()];
                }
                let bounds = at(ui, 0., 20., 590., 320.);
                ui.painter().rect_filled(bounds, 0., Color32::WHITE);
                ui.painter().rect_stroke(
                    bounds,
                    0.,
                    Stroke::new(1.0_f32, Color32::from_gray(140)),
                    StrokeKind::Inside,
                );
                ui.scope_builder(UiBuilder::new().max_rect(bounds.shrink(2.)), |ui| {
                    let columns = [
                        ("Frequency [MHz]", 76.),
                        ("S11 Mag", 65.),
                        ("S11 Angle", 65.),
                        ("S21 Mag", 65.),
                        ("S21 Angle", 65.),
                        ("S12 Mag", 65.),
                        ("S12 Angle", 65.),
                        ("S22 Mag", 65.),
                        ("S22 Angle", 65.),
                    ];
                    let rows: Vec<_> = n
                        .points
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let mut cells = vec![table::Cell::Text(format!("{:.2}", p.hz / 1e6))];
                            for s in p.s {
                                cells.push(format!("{:.2}", s.norm()).into());
                                cells.push(format!("{:.2}", s.arg().to_degrees()).into());
                            }
                            (i, cells)
                        })
                        .collect();
                    let mut selected = a
                        .import_selection
                        .iter()
                        .enumerate()
                        .filter(|(_, v)| **v)
                        .map(|(i, _)| i)
                        .collect();
                    table::show(ui, "import_parameters", &columns, &rows, &mut selected);
                    a.import_selection.fill(false);
                    for i in selected {
                        a.import_selection[i] = true;
                        a.selected_s = i;
                    }
                });
                if button(ui, 292., 358., 89., 22., "Select File") {
                    a.open_network_dialog();
                    if let Some(n) = &a.p.network {
                        a.import_selection = vec![false; n.points.len()];
                    }
                }
                let (ok, cancel, help) = footer(ui, 358., 590.);
                if cancel {
                    a.import_dialog = None;
                    a.classic.import_for_circle = false;
                }
                if help {
                    a.classic.help_topic = 39;
                    a.help = true;
                }
                if ok {
                    if a.classic.import_for_circle {
                        let i = a
                            .import_selection
                            .iter()
                            .position(|v| *v)
                            .unwrap_or(a.selected_s)
                            .min(n.points.len() - 1);
                        a.circle_s = n.points[i].s;
                        a.classic.circle_frequency = Some(n.points[i].hz);
                        if let Some(noise) = n.noise.iter().min_by(|x, y| {
                            (x.hz - n.points[i].hz)
                                .abs()
                                .total_cmp(&(y.hz - n.points[i].hz).abs())
                        }) {
                            a.noise_values = [
                                noise.fmin_db + 0.1,
                                noise.fmin_db,
                                noise.opt.norm(),
                                noise.opt.arg().to_degrees(),
                                noise.rn,
                            ];
                        }
                        a.classic.import_for_circle = false;
                        a.mode = false;
                    } else {
                        a.checkpoint();
                        for (i, p) in n.points.iter().enumerate() {
                            if a.import_selection[i] {
                                a.p.points.push(Datapoint {
                                    name: format!("DP{}", a.p.points.len() + 1),
                                    z: impedance(p.s[if port == 0 { 0 } else { 3 }], n.z0),
                                    hz: p.hz,
                                });
                            }
                        }
                        a.mode = false;
                    }
                    a.import_dialog = None;
                }
            },
        );
        if close {
            self.import_dialog = None;
            self.classic.import_for_circle = false;
        }
    }
    fn copy_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(
            ctx,
            "copy",
            "Copy to Clipboard",
            vec2(640., 290.),
            true,
            |a, ui| {
                group(ui, 4., 11., 307., 190., "Smith Chart");
                group(ui, 320., 11., 305., 190., "SPlot");
                mini_chart(ui, at(ui, 10., 22., 153., 168.), &a.p);
                for (i, name) in ["Smith Chart", "Schematic", "Datapoints", "Circles"]
                    .iter()
                    .enumerate()
                {
                    let (x, y) = if i == 0 {
                        (39., 100.)
                    } else {
                        (174., 21. + (i - 1) as f32 * 48.)
                    };
                    if radio(ui, x, y, 125., a.classic.copy_item == i, name) {
                        a.classic.copy_item = i;
                    }
                }
                for (i, name) in ["S11", "S12", "S21", "S22"].iter().enumerate() {
                    let x = 325. + (i % 2) as f32 * 78.;
                    let y = 22. + (i / 2) as f32 * 84.;
                    mini_chart(ui, at(ui, x, y, 75., 80.), &Project::default());
                    if radio(ui, x + 8., y + 35., 62., a.classic.copy_item == i + 4, name) {
                        a.classic.copy_item = i + 4;
                    }
                }
                for (i, name) in ["File info", "Gain table", "Gain graph", "Return Loss graph"]
                    .iter()
                    .enumerate()
                {
                    if radio(
                        ui,
                        487.,
                        24. + i as f32 * 43.,
                        137.,
                        a.classic.copy_item == i + 8,
                        name,
                    ) {
                        a.classic.copy_item = i + 8;
                    }
                }
                let (ok, cancel, help) = footer(ui, 220., 625.);
                if cancel {
                    a.classic.copy_dialog = false;
                }
                if help {
                    a.classic.help_topic = 31;
                    a.help = true;
                }
                if ok {
                    let keys = [
                        "chart",
                        "schematic",
                        "datapoints",
                        "circles",
                        "s11",
                        "s12",
                        "s21",
                        "s22",
                        "source",
                        "gain_table",
                        "gain",
                        "return_loss",
                    ];
                    let key = keys[a.classic.copy_item];
                    a.mode = a.classic.copy_item >= 4;
                    if let Some(window) = a.classic.windows.get_mut(key) {
                        window.visible = true;
                        window.minimized = false;
                    }
                    a.classic.capture_pending = Some(key.into());
                    a.classic.copy_dialog = false;
                    ctx.request_repaint();
                }
            },
        );
        if close {
            self.classic.copy_dialog = false;
        }
    }
    fn print_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(ctx, "print", "Print", vec2(384., 294.), true, |a, ui| {
            group(ui, 7., 19., 188., 174., "Smith Chart");
            group(ui, 203., 19., 170., 174., "SPlot");
            let left = [
                ("Smith Chart", 0, 14., 28.),
                ("Z-Chart", 1, 35., 46.),
                ("Y-Chart", 2, 35., 62.),
                ("Circuit Elements", 3, 35., 78.),
                ("Q-Circles", 4, 35., 94.),
                ("VSWR-Circles", 5, 35., 110.),
                ("Stability-Circles", 6, 108., 46.),
                ("Constant Gain Circles", 7, 108., 62.),
                ("Noise-Circles", 8, 108., 78.),
                ("Labels", 9, 108., 94.),
                ("Sweep-lines", 10, 108., 110.),
                ("Schematic", 11, 14., 134.),
                ("Datapoints", 12, 14., 150.),
                ("Circles", 13, 14., 166.),
            ];
            for (text, i, x, y) in left {
                enabled_at(ui, !a.mode, |ui| {
                    checkbox(
                        ui,
                        x,
                        y,
                        if x > 100. {
                            83.
                        } else if x > 30. {
                            73.
                        } else {
                            125.
                        },
                        &mut a.classic.print_items[i],
                        text,
                    );
                });
            }
            for (i, text) in [
                "S11",
                "S12",
                "S21",
                "S22",
                "File info",
                "Gain table",
                "Gain graph",
                "Return Loss graph",
            ]
            .iter()
            .enumerate()
            {
                enabled_at(ui, a.mode, |ui| {
                    let (column, row) = [
                        (0, 0),
                        (1, 0),
                        (0, 1),
                        (1, 1),
                        (0, 2),
                        (0, 3),
                        (1, 2),
                        (1, 3),
                    ][i];
                    checkbox(
                        ui,
                        211. + column as f32 * 78.,
                        46. + row as f32 * 16.,
                        77.,
                        &mut a.classic.print_items[i + 14],
                        text,
                    );
                });
            }
            let (ok, cancel, help) = footer(ui, 210., 378.);
            if cancel {
                a.classic.print_dialog = false;
            }
            if help {
                a.classic.help_topic = 30;
                a.help = true;
            }
            if ok {
                a.classic.print_dialog = false;
                a.print_report(ctx);
            }
        });
        if close {
            self.classic.print_dialog = false;
        }
    }
    fn preview_form(&mut self, ctx: &egui::Context) {
        let pages = crate::printing::pages(&self.p, self.mode, self.classic.print_items);
        let close = self.modal(
            ctx,
            "print_preview",
            "Print Preview",
            vec2(270., 245.),
            true,
            |a, ui| {
                if button(ui, 0., 0., 36., 19., "Print") {
                    let was = a.classic.print_preview;
                    a.classic.print_preview = false;
                    a.print_report(ctx);
                    a.classic.print_preview = was;
                }
                if button(ui, 39., 0., 18., 19., "‹") {
                    a.classic.preview_page = a.classic.preview_page.saturating_sub(1);
                }
                if button(ui, 59., 0., 18., 19., "›") {
                    a.classic.preview_page =
                        (a.classic.preview_page + 1).min(pages.len().saturating_sub(1));
                }
                if button(ui, 81., 0., 23., 19., "−") {
                    a.classic.preview_zoom = a.classic.preview_zoom.saturating_sub(1);
                }
                if button(ui, 106., 0., 23., 19., "+") {
                    a.classic.preview_zoom = (a.classic.preview_zoom + 1).min(4);
                }
                combo(
                    ui,
                    "preview_zoom",
                    134.,
                    0.,
                    98.,
                    &mut a.classic.preview_zoom,
                    &["Whole page", "Page width", "50%", "100%", "200%"],
                );
                let area = at(ui, 0., 24., 260., 162.);
                ui.painter().rect_filled(area, 0., DESKTOP);
                if let Some(page) =
                    pages.get(a.classic.preview_page.min(pages.len().saturating_sub(1)))
                {
                    let scale = match a.classic.preview_zoom {
                        0 => 150. / 842.,
                        1 => 246. / 595.,
                        2 => 0.5,
                        3 => 1.,
                        _ => 2.,
                    };
                    ui.scope_builder(UiBuilder::new().max_rect(area), |ui| {
                        ui.set_clip_rect(area);
                        egui::ScrollArea::both().show(ui, |ui| {
                            let size = vec2(595. * scale, 842. * scale);
                            let (r, _) = ui.allocate_exact_size(
                                vec2((size.x + 12.).max(260.), size.y + 12.),
                                Sense::hover(),
                            );
                            let paper = Rect::from_min_size(
                                pos2(r.center().x - size.x / 2., r.top() + 6.),
                                size,
                            );
                            crate::printing::preview(ui, page, paper);
                        });
                    });
                    label(
                        ui,
                        3.,
                        188.,
                        255.,
                        format!(
                            "{} / {}    {}",
                            a.classic.preview_page + 1,
                            pages.len(),
                            page.title
                        ),
                    );
                } else {
                    label(ui, 5., 90., 250., "No items selected for printing");
                }
            },
        );
        if close {
            self.classic.preview_open = false;
        }
    }
    fn export_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(
            ctx,
            "export_data",
            if self.classic.export_circles {
                "Export Circles"
            } else {
                "Export Datapoints"
            },
            vec2(335., 168.),
            true,
            |a, ui| {
                group(ui, 8., 13., 310., 54., "Delimiter");
                for (i, text) in ["None", "Semicolon", "Comma"].iter().enumerate() {
                    if radio(
                        ui,
                        18. + i as f32 * 100.,
                        29.,
                        94.,
                        a.classic.delimiter == i,
                        text,
                    ) {
                        a.classic.delimiter = i;
                    }
                }
                let (ok, cancel, help) = footer(ui, 97., 318.);
                if cancel {
                    a.classic.export_dialog = false;
                }
                if help {
                    a.classic.help_topic = 34;
                    a.help = true;
                }
                if ok {
                    let delimiter = [' ', ';', ','][a.classic.delimiter];
                    let text = if a.classic.export_circles {
                        formats::circle_text(&a.p, delimiter)
                    } else {
                        formats::point_text(&a.p, delimiter)
                    };
                    a.export("txt", text);
                    a.classic.export_dialog = false;
                }
            },
        );
        if close {
            self.classic.export_dialog = false;
        }
    }
    fn parameter_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(
            ctx,
            "export_parameters",
            "Export Parameters",
            vec2(330., 230.),
            true,
            |a, ui| {
                group(ui, 8., 13., 306., 52., "Twoport Parameter");
                for (i, name) in ["H", "Z", "Y", "A"].iter().enumerate() {
                    if radio(
                        ui,
                        19. + i as f32 * 72.,
                        29.,
                        65.,
                        a.export_kind == *name,
                        name,
                    ) {
                        a.export_kind = (*name).into();
                    }
                }
                group(ui, 8., 81., 306., 51., "Normalization");
                if radio(ui, 19., 96., 132., a.normalized, "Normalized") {
                    a.normalized = true;
                }
                if radio(ui, 161., 96., 140., !a.normalized, "Not normalized") {
                    a.normalized = false;
                }
                let (ok, cancel, help) = footer(ui, 163., 314.);
                if cancel {
                    a.classic.parameter_dialog = false;
                }
                if help {
                    a.classic.help_topic = 37;
                    a.help = true;
                }
                if ok {
                    if let Some(n) = &a.p.network {
                        match formats::export_parameters(n, &a.export_kind, a.normalized) {
                            Ok(text) => a.export(if n.ports == 1 { "s1p" } else { "s2p" }, text),
                            Err(e) => a.status = e,
                        }
                    }
                    a.classic.parameter_dialog = false;
                }
            },
        );
        if close {
            self.classic.parameter_dialog = false;
        }
    }
    fn about_form(&mut self, ctx: &egui::Context) {
        let close = self.modal(
            ctx,
            "about",
            "About Smith",
            vec2(430., 330.),
            true,
            |a, ui| {
                ui.painter().text(
                    at(ui, 0., 26., 416., 35.).center(),
                    Align2::CENTER_CENTER,
                    "Smith V4.1",
                    FontId::proportional(26.),
                    Color32::from_rgb(0, 0, 160),
                );
                mini_chart(ui, at(ui, 17., 76., 128., 128.), &Project::default());
                label(ui, 166., 79., 237., "SmithCraft — Rust implementation");
                label(
                    ui,
                    166.,
                    105.,
                    237.,
                    concat!("Version ", env!("CARGO_PKG_VERSION")),
                );
                label(ui, 166., 136., 237., "Reference interface: Smith V4.1");
                label(ui, 166., 156., 237., "Fritz Dellsperger / BFH");
                label(ui, 166., 183., 237., "Independent implementation in Rust");
                if action(ui, 350., 253., 60., "OK", 0) {
                    a.classic.about = false;
                }
            },
        );
        if close {
            self.classic.about = false;
        }
    }
    fn help_form(&mut self, _ctx: &egui::Context) {
        let topic = self.classic.help_topic.max(1);
        if let Err(e) = crate::help::open(topic) {
            self.status = e;
        }
        self.help = false;
    }
}
fn mini_chart(ui: &mut Ui, r: Rect, project: &Project) {
    let p = ui.painter().with_clip_rect(r);
    p.rect_filled(r, 0., Color32::WHITE);
    let center = r.center();
    let radius = r.width().min(r.height()) * 0.46;
    let pos = |g: C| center + vec2(g.re as f32 * radius, -g.im as f32 * radius);
    p.circle_stroke(center, radius, Stroke::new(0.6f32, Color32::from_gray(140)));
    for (sign, color) in [
        (1., Color32::from_rgb(220, 170, 170)),
        (-1., Color32::from_rgb(160, 200, 220)),
    ] {
        for v in [0.2, 0.5, 1., 2., 5.] {
            let pts = (0..=150)
                .map(|i| {
                    let a = TAU * i as f64 / 150.;
                    pos(c(sign * v / (v + 1.), 0.) + C::from_polar(1. / (v + 1.), a))
                })
                .collect();
            p.add(egui::Shape::line(pts, Stroke::new(0.5f32, color)));
        }
    }
    if let Some(load) = project.active() {
        let mut z = load.z;
        for e in &project.elements {
            let pts = (0..=60)
                .map(|i| {
                    pos(gamma(
                        e.apply_fraction(z, load.hz, i as f64 / 60.),
                        project.z0,
                    ))
                })
                .collect();
            p.add(egui::Shape::line(
                pts,
                Stroke::new(1.0_f32, Color32::from_rgb(120, 40, 150)),
            ));
            z = e.apply(z, load.hz);
        }
    }
}
