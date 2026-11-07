use super::*;
#[derive(Clone, Copy)]
enum Icon {
    Open,
    Save,
    Print,
    Undo,
    Redo,
    Copy,
    ZoomIn,
    ZoomOut,
    One,
    Mode,
    Circles,
    Settings,
    Element(Kind),
}
fn icon(ui: &Ui, r: Rect, kind: Icon) {
    let p = ui.painter();
    let c = r.center();
    let pen = Stroke::new(1.0_f32, Color32::from_rgb(35, 70, 90));
    match kind {
        Icon::Element(kind) => {
            let e = Element::new(kind, 1e9);
            crate::schematic::toolbar_symbol(p, r, &e);
        }
        Icon::Open => {
            p.rect_filled(
                Rect::from_center_size(c, vec2(11., 8.)),
                0.,
                Color32::from_rgb(236, 202, 80),
            );
            p.line_segment([c + vec2(-6., -4.), c + vec2(-1., -6.)], pen);
            p.rect_filled(
                Rect::from_center_size(c + vec2(2., 1.), vec2(8., 6.)),
                0.,
                Color32::from_rgb(82, 155, 195),
            );
        }
        Icon::Save => {
            p.rect_filled(
                Rect::from_center_size(c, vec2(10., 10.)),
                0.,
                Color32::from_rgb(77, 84, 145),
            );
            p.rect_filled(
                Rect::from_center_size(c + vec2(0., 2.), vec2(6., 4.)),
                0.,
                Color32::WHITE,
            );
            p.rect_filled(
                Rect::from_center_size(c - vec2(0., 3.), vec2(4., 3.)),
                0.,
                Color32::from_gray(190),
            );
        }
        Icon::Print => {
            p.rect_filled(
                Rect::from_center_size(c, vec2(11., 6.)),
                0.,
                Color32::from_rgb(139, 169, 166),
            );
            p.rect_stroke(
                Rect::from_center_size(c, vec2(11., 6.)),
                0.,
                pen,
                StrokeKind::Inside,
            );
            p.rect_filled(
                Rect::from_center_size(c + vec2(0., 4.), vec2(7., 4.)),
                0.,
                Color32::WHITE,
            );
        }
        Icon::Undo | Icon::Redo => {
            let sign = if matches!(kind, Icon::Redo) { -1. } else { 1. };
            let pts = (0..=18)
                .map(|i| {
                    let a = -2.5 + i as f32 / 18. * 3.;
                    c + vec2(a.cos() * 5. * sign, a.sin() * 4.)
                })
                .collect();
            p.add(egui::Shape::line(
                pts,
                Stroke::new(1.5f32, Color32::from_rgb(75, 154, 174)),
            ));
            p.arrow(c + vec2(-3. * sign, -1.), vec2(-3. * sign, -3.), pen);
        }
        Icon::Copy => {
            p.rect_filled(Rect::from_center_size(c, vec2(9., 11.)), 0., Color32::WHITE);
            p.rect_stroke(
                Rect::from_center_size(c, vec2(9., 11.)),
                0.,
                pen,
                StrokeKind::Inside,
            );
            p.rect_filled(
                Rect::from_center_size(c - vec2(0., 5.), vec2(5., 3.)),
                0.,
                Color32::from_gray(170),
            );
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            p.circle_stroke(
                c - vec2(2., 2.),
                5.,
                Stroke::new(1.2f32, Color32::from_gray(130)),
            );
            p.line_segment(
                [c + vec2(2., 2.), c + vec2(7., 7.)],
                Stroke::new(2.0_f32, Color32::from_gray(150)),
            );
            p.line_segment([c + vec2(-5., -2.), c + vec2(1., -2.)], pen);
            if matches!(kind, Icon::ZoomIn) {
                p.line_segment([c + vec2(-2., -5.), c + vec2(-2., 1.)], pen);
            }
        }
        Icon::One => {
            p.text(
                c,
                Align2::CENTER_CENTER,
                "1:1",
                FontId::proportional(11.),
                Color32::from_gray(80),
            );
        }
        Icon::Mode => {
            for y in -2..=2 {
                let py = c.y + y as f32 * 2.;
                p.line_segment(
                    [pos2(c.x - 7., py), pos2(c.x + 7., py)],
                    Stroke::new(0.5f32, Color32::from_rgb(0, 130, 190)),
                );
            }
            p.text(
                c + vec2(0., 5.),
                Align2::CENTER_CENTER,
                "S↔",
                FontId::proportional(8.),
                Color32::GREEN,
            );
        }
        Icon::Circles => {
            for rad in [2., 5., 8.] {
                p.circle_stroke(c, rad, Stroke::new(1.0_f32, Color32::from_rgb(200, 45, 60)));
            }
            p.line_segment([c - vec2(9., 0.), c + vec2(9., 0.)], pen);
        }
        Icon::Settings => {
            for i in 0..10 {
                let a = i as f32 * std::f32::consts::TAU / 10.;
                p.line_segment(
                    [
                        c + vec2(a.cos(), a.sin()) * 4.,
                        c + vec2(a.cos(), a.sin()) * 8.,
                    ],
                    Stroke::new(2.5f32, Color32::from_gray(140)),
                );
            }
            p.circle_filled(c, 5., Color32::from_gray(180));
            p.circle_stroke(c, 2., pen);
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn tool(
    ui: &mut Ui,
    r: Rect,
    id: impl std::hash::Hash,
    kind: Option<Icon>,
    text: &str,
    tip: &str,
    enabled: bool,
    selected: bool,
) -> bool {
    let response = ui.interact(
        r,
        ui.make_persistent_id(id),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if selected || response.hovered() && enabled {
        gradient(
            ui.painter(),
            r.shrink(1.),
            if selected {
                Color32::from_rgb(145, 235, 153)
            } else {
                Color32::from_rgb(236, 248, 255)
            },
            Color32::from_rgb(174, 205, 234),
        );
        ui.painter().rect_stroke(
            r.shrink(1.),
            0.,
            Stroke::new(1.0_f32, Color32::from_rgb(105, 155, 198)),
            StrokeKind::Inside,
        );
    }
    if let Some(kind) = kind {
        icon(ui, r, kind);
    } else {
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(10.),
            Color32::from_gray(if enabled { 20 } else { 135 }),
        );
    }
    response.on_hover_text(tip).clicked() && enabled
}
fn item(ui: &mut Ui, text: &str, shortcut: &str, enabled: bool) -> bool {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(format!("  {text}")).shortcut_text(shortcut),
    );
    let kind = match text {
        "Open Project" | "Open S-Parameter" => Some(Icon::Open),
        "Save Project" | "Save Project As" => Some(Icon::Save),
        "Print" => Some(Icon::Print),
        "Undo" => Some(Icon::Undo),
        "Redo" => Some(Icon::Redo),
        "Copy to Clipboard" => Some(Icon::Copy),
        _ => None,
    };
    if let Some(kind) = kind {
        icon(
            ui,
            Rect::from_center_size(response.rect.left_center() + vec2(6., 0.), vec2(13., 13.)),
            kind,
        );
    }
    if response.clicked() {
        ui.close();
    }
    response.clicked()
}
impl SmithApp {
    pub(in super::super) fn begin_tune(&mut self) {
        self.tune_centers = self.p.elements.iter().map(|e| e.value).collect();
        self.tune_z0_centers = self.p.elements.iter().map(|e| e.z0).collect();
        self.tune_lc_centers = self.p.elements.iter().map(|e| [e.l_nh, e.c_pf]).collect();
        self.tune = true;
    }
    pub(in super::super) fn show_settings(&mut self) {
        self.classic.style_project = None;
        self.classic.style_backup = None;
        self.settings = true;
    }
    pub(in super::super) fn rearrange(&mut self) {
        self.classic.windows.clear();
        self.classic.initialized = false;
        self.classic.layout_frames = 0;
        self.classic.scroll = [0.; 2];
        self.classic.active = if self.mode { "s11" } else { "chart" }.into();
    }
    pub(in super::super) fn new_project(&mut self) {
        if self.modified() {
            self.navigation = Some(Navigation::New);
            self.closing = true;
        } else {
            self.reset_project();
        }
    }
    pub(in super::super) fn reset_project(&mut self) {
        self.history.clear();
        self.p = Project {
            z0: self.p.z0,
            settings: self.p.settings.clone(),
            default_hz: self.p.default_hz,
            ..Project::default()
        };
        self.saved = self.p.clone();
        self.path = None;
        self.pending = None;
        self.mouse = false;
        self.zoom = 1.;
        self.status.clear();
    }
    pub(in super::super) fn classic_menus(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("classic_menus")
            .exact_height(30.)
            .frame(
                egui::Frame::NONE
                    .fill(FORM)
                    .inner_margin(egui::Margin::symmetric(5, 0)),
            )
            .show(ctx, |ui| {
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Button, FontId::proportional(18.));
                ui.spacing_mut().button_padding = vec2(10., 2.);
                egui::MenuBar::new()
                    .config(egui::containers::menu::MenuConfig::new().style(
                        |style: &mut egui::Style| {
                            egui::containers::menu::menu_style(style);
                            style
                                .text_styles
                                .insert(egui::TextStyle::Button, FontId::proportional(18.));
                            style.spacing.interact_size.y = 24.;
                            style.spacing.button_padding = vec2(8., 2.);
                            style.spacing.item_spacing.y = 2.;
                        },
                    ))
                    .style(|style: &mut egui::Style| {
                        egui::containers::menu::menu_style(style);
                        style.spacing.button_padding = vec2(10., 2.);
                    })
                    .ui(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            ui.set_min_width(222.);
                            if item(ui, "New", "CTRL+N", true) {
                                self.new_project();
                            }
                            ui.add(egui::Separator::default().spacing(0.));
                            if item(
                                ui,
                                if self.mode {
                                    "Open S-Parameter"
                                } else {
                                    "Open Project"
                                },
                                "CTRL+O",
                                true,
                            ) {
                                self.open();
                            }
                            ui.add(egui::Separator::default().spacing(0.));
                            if item(ui, "Save Project", "CTRL+S", !self.mode) {
                                self.save_as(false);
                            }
                            if item(ui, "Save Project As", "", !self.mode) {
                                self.save_as(true);
                            }
                            if item(ui, "Save Netlist", "", !self.mode) {
                                self.export("ckt", formats::netlist(&self.p));
                            }
                            ui.add(egui::Separator::default().spacing(0.));
                            if !self.mode {
                                ui.menu_button("Export", |ui| {
                                    if item(ui, "Datapoints", "", true) {
                                        self.classic.export_circles = false;
                                        self.classic.export_dialog = true;
                                    }
                                    if item(ui, "Circles", "", true) {
                                        self.classic.export_circles = true;
                                        self.classic.export_dialog = true;
                                    }
                                    if item(ui, "Smith Chart (SVG)", "", true) {
                                        self.export("svg", chart::svg(&self.p));
                                    }
                                });
                            }
                            ui.add(egui::Separator::default().spacing(0.));
                            if item(ui, "Print", "", true) {
                                self.classic.print_dialog = true;
                                self.classic.print_preview = false;
                            }
                            if item(ui, "Print Preview", "", true) {
                                self.classic.print_dialog = true;
                                self.classic.print_preview = true;
                            }
                            ui.add(egui::Separator::default().spacing(0.));
                            ui.menu_button("Recent Files", |ui| {
                                if self.classic.recent.is_empty() {
                                    ui.add_enabled(false, egui::Label::new("(empty)"));
                                }
                                for path in self.classic.recent.clone() {
                                    if item(ui, &path.to_string_lossy(), "", true) {
                                        self.open_path(path);
                                    }
                                }
                            });
                            ui.add(egui::Separator::default().spacing(0.));
                            if item(ui, "Exit", "", true) {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if item(ui, "Undo", "CTRL+Z", self.history.can_undo()) {
                                self.history.undo(&mut self.p);
                            }
                            if item(ui, "Redo", "CTRL+Y", self.history.can_redo()) {
                                self.history.redo(&mut self.p);
                            }
                            ui.add(egui::Separator::default().spacing(0.));
                            if item(ui, "Copy to Clipboard", "", true) {
                                self.classic.copy_dialog = true;
                            }
                        });
                        ui.menu_button("Mode", |ui| {
                            if ui
                                .selectable_label(!self.mode, "Smith Chart     Ctrl+M")
                                .clicked()
                            {
                                self.mode = false;
                                ui.close();
                            }
                            if ui
                                .selectable_label(self.mode, "S-Plot              Ctrl+M")
                                .clicked()
                            {
                                self.mode = true;
                                ui.close();
                            }
                        });
                        if self.mode {
                            ui.menu_button("Export", |ui| {
                                for kind in ["H", "Z", "Y", "A"] {
                                    if item(
                                        ui,
                                        &format!("s- to {kind}-parameter - file"),
                                        "",
                                        self.p.network.as_ref().is_some_and(|n| n.ports == 2),
                                    ) {
                                        self.export_kind = kind.into();
                                        self.classic.parameter_dialog = true;
                                    }
                                }
                                if item(ui, "CITI- to S", "", self.p.network.is_some())
                                    && let Some(n) = &self.p.network
                                {
                                    match formats::export_parameters(n, "S", false) {
                                        Ok(text) => self
                                            .export(if n.ports == 1 { "s1p" } else { "s2p" }, text),
                                        Err(e) => self.status = e,
                                    }
                                }
                                ui.add(egui::Separator::default().spacing(0.));
                                if item(ui, "s11 to Smith Chart", "", self.p.network.is_some()) {
                                    self.choose_import(0);
                                }
                                if item(
                                    ui,
                                    "s22 to Smith Chart",
                                    "",
                                    self.p.network.as_ref().is_some_and(|n| n.ports == 2),
                                ) {
                                    self.choose_import(1);
                                }
                            });
                        }
                        if !self.mode {
                            ui.menu_button("Tools", |ui| {
                                if item(ui, "Circles", "CTRL+T", !self.mode) {
                                    self.circle_dialog = true;
                                }
                                ui.add(egui::Separator::default().spacing(0.));
                                if item(ui, "Sweep", "", !self.mode) {
                                    self.sweep_dialog = true;
                                }
                                if item(ui, "Remove Sweep", "", self.p.sweep.visible && !self.mode)
                                {
                                    self.checkpoint();
                                    self.p.sweep.visible = false;
                                }
                                ui.add(egui::Separator::default().spacing(0.));
                                if item(ui, "Tune", "", !self.mode) {
                                    self.begin_tune();
                                }
                                ui.add(egui::Separator::default().spacing(0.));
                                if item(ui, "Settings", "CTRL+E", true) {
                                    self.show_settings();
                                }
                            });
                            ui.menu_button("Zoom", |ui| {
                                if item(ui, "Zoom in", "CTRL+1", true) {
                                    self.zoom = (self.zoom * 1.25).min(8.);
                                }
                                if item(ui, "Zoom out", "CTRL+2", true) {
                                    self.zoom = (self.zoom / 1.25).max(0.1);
                                }
                                if item(ui, "Zoom 1:1", "CTRL+3", self.zoom != 1.) {
                                    self.zoom = 1.;
                                }
                            });
                        }
                        ui.menu_button("Window", |ui| {
                            if item(ui, "Rearrange all", "", true) {
                                self.rearrange();
                            }
                        });
                        ui.menu_button("Help", |ui| {
                            if item(ui, "Help Smith", "CTRL+F1", true) {
                                self.classic.help_topic = 0;
                                self.help = true;
                            }
                            if item(ui, "About Smith", "", true) {
                                self.classic.about = true;
                            }
                        });
                    });
            });
        egui::TopBottomPanel::top("classic_toolbar")
            .exact_height(28.)
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let bounds = ui.max_rect();
                gradient(
                    ui.painter(),
                    bounds,
                    Color32::from_rgb(194, 222, 255),
                    Color32::from_rgb(181, 211, 247),
                );
                let groups = [
                    ("Standard", 0., 110.),
                    ("Datapoints", 114., 160.),
                    ("Zoom", 278., 75.),
                    ("Mode", 355., 155.),
                    ("Elements", 507., 327.),
                ];
                for (name, x, w) in groups {
                    if self.mode && matches!(name, "Datapoints" | "Zoom" | "Elements") {
                        continue;
                    }
                    if self.classic.toolbar_positions.contains_key(name) {
                        continue;
                    }
                    let r = Rect::from_min_size(bounds.min + vec2(x, 0.), vec2(w, 27.));
                    self.toolbar_group(ui, r, name);
                }
            });
        for (name, pos) in self.classic.toolbar_positions.clone() {
            if self.mode && matches!(name.as_str(), "Datapoints" | "Zoom" | "Elements") {
                continue;
            }
            egui::Area::new(Id::new(("floating_toolbar", &name)))
                .fixed_pos(pos2(pos[0], pos[1]))
                .order(egui::Order::Foreground)
                .movable(false)
                .show(ctx, |ui| {
                    let w = match name.as_str() {
                        "Standard" => 110.,
                        "Datapoints" => 160.,
                        "Zoom" => 75.,
                        "Mode" => 155.,
                        _ => 327.,
                    };
                    ui.set_min_size(vec2(w, 27.));
                    let r = Rect::from_min_size(ui.min_rect().min, vec2(w, 27.));
                    gradient(
                        ui.painter(),
                        r,
                        Color32::from_rgb(194, 222, 255),
                        Color32::from_rgb(181, 211, 247),
                    );
                    self.toolbar_group(ui, r, &name);
                });
        }
    }
    fn toolbar_group(&mut self, ui: &mut Ui, r: Rect, name: &str) {
        let grip = Rect::from_min_size(r.min, vec2(7., 27.));
        for y in (4..24).step_by(4) {
            ui.painter().rect_filled(
                Rect::from_min_size(grip.min + vec2(2., y as f32), vec2(2., 1.)),
                0.,
                Color32::from_rgb(112, 148, 179),
            );
        }
        let response = ui.interact(
            grip,
            ui.make_persistent_id((name, "grip")),
            Sense::click_and_drag(),
        );
        if response.dragged() {
            let current = self
                .classic
                .toolbar_positions
                .get(name)
                .copied()
                .unwrap_or([r.min.x, r.min.y]);
            let delta = response.drag_delta();
            self.classic
                .toolbar_positions
                .insert(name.into(), [current[0] + delta.x, current[1] + delta.y]);
        }
        if response.double_clicked() {
            self.classic.toolbar_positions.remove(name);
        }
        let mut x = r.left() + 7.;
        let y = r.top() + 2.;
        let mut b = |icon_kind, text, tip, width, enabled, selected| {
            let rect = Rect::from_min_size(pos2(x, y), vec2(width, 23.));
            x += width;
            tool(
                ui,
                rect,
                (name, tip),
                icon_kind,
                text,
                tip,
                enabled,
                selected,
            )
        };
        match name {
            "Standard" => {
                if b(Some(Icon::Open), "", "Open project", 17., true, false) {
                    self.open();
                }
                if b(Some(Icon::Save), "", "Save project", 17., !self.mode, false) {
                    self.save_as(false);
                }
                if b(Some(Icon::Print), "", "Print", 17., true, false) {
                    self.classic.print_dialog = true;
                    self.classic.print_preview = false;
                }
                if b(
                    Some(Icon::Undo),
                    "",
                    "Undo",
                    17.,
                    self.history.can_undo(),
                    false,
                ) {
                    self.history.undo(&mut self.p);
                }
                if b(
                    Some(Icon::Redo),
                    "",
                    "Redo",
                    17.,
                    self.history.can_redo(),
                    false,
                ) {
                    self.history.redo(&mut self.p);
                }
                if b(Some(Icon::Copy), "", "Copy to clipboard", 17., true, false) {
                    self.classic.copy_dialog = true;
                }
            }
            "Datapoints" => {
                if b(
                    None,
                    "Mouse",
                    "Input datapoints with mouse",
                    40.,
                    !self.mode,
                    self.mouse,
                ) {
                    self.mouse = !self.mouse;
                    self.pending = None;
                }
                if b(None, "Key…", "Keyboard input", 39., !self.mode, false) {
                    self.add_point_dialog(None);
                }
                if b(None, "S1", "Import S11", 21., !self.mode, false) {
                    self.choose_import(0);
                }
                if b(None, "S2", "Import S22", 21., !self.mode, false) {
                    self.choose_import(1);
                }
            }
            "Zoom" => {
                if b(Some(Icon::ZoomIn), "", "Zoom in", 23., true, false) {
                    self.zoom = (self.zoom * 1.25).min(8.);
                }
                if b(Some(Icon::ZoomOut), "", "Zoom out", 23., true, false) {
                    self.zoom = (self.zoom / 1.25).max(0.1);
                }
                if b(Some(Icon::One), "", "Zoom 1:1", 23., self.zoom != 1., false) {
                    self.zoom = 1.;
                }
            }
            "Mode" => {
                if b(
                    Some(Icon::Mode),
                    "",
                    "Toggle Smith Chart / S-Plot",
                    25.,
                    true,
                    false,
                ) {
                    self.mode = !self.mode;
                }
                if b(Some(Icon::Circles), "", "Circles", 25., !self.mode, false) {
                    self.circle_dialog = true;
                }
                if b(Some(Icon::Settings), "", "Settings", 25., true, false) {
                    self.show_settings();
                }
                if b(None, "S…", "Sweep", 26., !self.mode, false) {
                    self.sweep_dialog = true;
                }
                if b(
                    None,
                    "Clr",
                    "Clear sweep",
                    26.,
                    !self.mode && self.p.sweep.visible,
                    false,
                ) {
                    self.checkpoint();
                    self.p.sweep.visible = false;
                }
                if b(None, "Tu", "Tune", 21., !self.mode, false) {
                    self.begin_tune();
                }
            }
            _ => {
                for kind in crate::schematic::TOOLBAR_KINDS {
                    if b(
                        Some(Icon::Element(kind)),
                        "",
                        kind.label(),
                        40. / 1.5,
                        !self.mode && self.p.active().is_some(),
                        false,
                    ) {
                        self.element(kind, None);
                    }
                }
            }
        }
    }
    pub(in super::super) fn print_report(&mut self, ctx: &egui::Context) {
        if self.classic.print_preview {
            self.classic.preview_open = true;
            self.classic.preview_page = 0;
            ctx.request_repaint();
            return;
        }
        let pages = crate::printing::pages(&self.p, self.mode, self.classic.print_items);
        self.status = match crate::printing::print(&pages) {
            Ok(()) => "Print dialog closed".into(),
            Err(e) => e,
        };
    }
}
