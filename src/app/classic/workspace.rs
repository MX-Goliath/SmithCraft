use super::*;
type GraphCurve<'a> = (&'a str, usize, Vec<(f64, f64)>);
fn frequency(hz: f64) -> String {
    if hz >= 1e9 {
        format!("{:.2}GHz", hz / 1e9)
    } else if hz >= 1e6 {
        format!("{:.2}MHz", hz / 1e6)
    } else if hz >= 1e3 {
        format!("{:.2}kHz", hz / 1e3)
    } else {
        format!("{hz:.2}Hz")
    }
}
fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}
fn path(p: &egui::Painter, points: Vec<Pos2>, color: Color32, width: f32, dashed: bool) {
    if points.len() < 2 {
        return;
    }
    if dashed {
        p.extend(egui::Shape::dashed_line(
            &points,
            Stroke::new(width, color),
            7.,
            4.,
        ));
    } else {
        p.add(egui::Shape::line(points, Stroke::new(width, color)));
    }
}
fn line_color(state: &ClassicState, i: usize) -> Color32 {
    rgb(state.color_rows[i])
}
fn rotated_text(p: &egui::Painter, pos: Pos2, text: String, angle: f32) {
    let galley = p.layout_no_wrap(text, FontId::proportional(10.), Color32::BLACK);
    p.add(egui::epaint::TextShape::new(pos, galley, Color32::BLACK).with_angle(angle));
}
struct Hit {
    response: egui::Response,
    g: Option<C>,
    radius: f32,
}
fn chart_view(
    ui: &mut Ui,
    project: &Project,
    state: &ClassicState,
    zoom: f32,
    network: Option<(&formats::Network, usize, usize)>,
) -> Hit {
    let size = ui.available_size();
    let (response, painter) = ui.allocate_painter(size, Sense::click_and_drag());
    let rect = response.rect;
    painter.rect_filled(rect, 0., Color32::WHITE);
    let expanded = network.is_none() && state.flags[9];
    let r = rect.width().min(rect.height()) * if network.is_some() { 0.445 } else { 0.475 } * zoom;
    let center = rect.center();
    let point = |g: C| center + vec2(g.re as f32 * r, -g.im as f32 * r);
    let font = FontId::new(
        if network.is_some() { 10. } else { 18. },
        egui::FontFamily::Name("chart".into()),
    );
    let polar = network.is_some_and(|(_, j, _)| j == 1 || j == 2);
    let mut scale = 1.;
    if polar {
        if let Some((n, j, _)) = network.filter(|(n, _, _)| !n.points.is_empty()) {
            scale = n
                .points
                .iter()
                .map(|p| p.s[j].norm())
                .fold(0f64, f64::max)
                .max(1e-12);
            let step = 10f64.powf(scale.log10().floor());
            scale = (scale * 1.1 / step).ceil() * step;
        } else {
            scale = 1.2;
        }
        for k in 1..=4 {
            let rr = r * k as f32 / 4.;
            path(
                &painter,
                (0..=240)
                    .map(|i| {
                        let a = TAU * i as f64 / 240.;
                        center + vec2(a.cos() as f32 * rr, a.sin() as f32 * rr)
                    })
                    .collect(),
                line_color(state, 16),
                state.widths[16] * 0.6,
                k != 4,
            );
            painter.text(
                center + vec2(3., -rr),
                Align2::LEFT_BOTTOM,
                format!("{:.2}", scale * k as f64 / 4.),
                FontId::proportional(9.),
                Color32::BLACK,
            );
        }
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.;
            painter.line_segment(
                [center, center + vec2(a.cos() * r, a.sin() * r)],
                Stroke::new(0.6f32, line_color(state, 16)),
            );
        }
    } else {
        let splot = network.is_some();
        let labels = if splot {
            state.flags[14]
        } else {
            project.settings.labels && state.flags[4]
        };
        for (enabled, sign, color, row) in [
            (
                if splot {
                    state.flags[12]
                } else {
                    project.settings.z_grid
                },
                1.,
                line_color(state, 0),
                0,
            ),
            (
                if splot {
                    state.flags[13]
                } else {
                    project.settings.y_grid
                },
                -1.,
                line_color(state, 1),
                1,
            ),
        ] {
            if !enabled {
                continue;
            }
            for resistance in [0.2, 0.5, 1., 2., 4., 10.] {
                let center_g = c(sign * resistance / (1. + resistance), 0.);
                let radius = 1. / (1. + resistance);
                let pts = (0..=480)
                    .map(|i| {
                        let a = TAU * i as f64 / 480.;
                        point(center_g + C::from_polar(radius, a))
                    })
                    .collect();
                path(
                    &painter,
                    pts,
                    if resistance == 1. {
                        line_color(state, row + 10)
                    } else {
                        color
                    },
                    if resistance == 1. {
                        state.widths[row + 10]
                    } else {
                        state.widths[row] * 0.55
                    },
                    resistance != 1.,
                );
                if sign < 0. && labels {
                    let label = if resistance == 0.2 {
                        format!("{:.2}m", resistance * 1000. / project.z0)
                    } else {
                        format!("{:.2}", resistance * 1000. / project.z0)
                    };
                    painter.text(
                        point(center_g + c(0., radius)) + vec2(3., 0.),
                        Align2::LEFT_CENTER,
                        label,
                        font.clone(),
                        color,
                    );
                }
                if sign > 0. && labels {
                    painter.text(
                        point(center_g - c(0., radius)) + vec2(-2., 0.),
                        Align2::RIGHT_CENTER,
                        format!("{:.2}", resistance * project.z0),
                        font.clone(),
                        color,
                    );
                }
            }
            for reactance in [-10., -4., -2., -1., -0.5, -0.2, 0.2, 0.5, 1., 2., 4., 10.] {
                let pts = (0..=400)
                    .map(|i| {
                        let g = if expanded {
                            c(1., 1. / reactance)
                                + C::from_polar(1. / reactance.abs(), TAU * i as f64 / 400.)
                        } else {
                            let resistance = 2000. * (i as f64 / 400.).powi(4);
                            gamma(c(resistance, reactance), 1.)
                        };
                        point(g * sign)
                    })
                    .collect();
                path(
                    &painter,
                    pts,
                    color,
                    (state.widths[row] * 0.55).max(0.5),
                    true,
                );
                if labels && reactance > 0. {
                    let g = gamma(c(0., reactance), 1.) * sign;
                    let value = if sign > 0. {
                        format!("{:.2}", reactance.abs() * project.z0)
                    } else if reactance == 0.2 {
                        format!("{:.2}m", reactance.abs() * 1000. / project.z0)
                    } else {
                        format!("{:.2}", reactance.abs() * 1000. / project.z0)
                    };
                    painter.text(
                        point(g * 1.035),
                        Align2::CENTER_CENTER,
                        value,
                        font.clone(),
                        color,
                    );
                }
            }
        }
        painter.line_segment(
            [point(c(-1., 0.)), point(c(1., 0.))],
            Stroke::new(0.6f32, line_color(state, 0)),
        );
        painter.circle_stroke(
            center,
            r,
            Stroke::new(state.widths[9] * 0.55, line_color(state, 9)),
        );
    }
    if let Some((n, j, selected)) = network {
        let points = n.points.iter().map(|s| point(s.s[j] / scale)).collect();
        path(
            &painter,
            points,
            line_color(state, 16),
            state.widths[16],
            false,
        );
        for (i, p) in n.points.iter().enumerate() {
            let pos = point(p.s[j] / scale);
            if i == selected {
                painter.rect_stroke(
                    Rect::from_center_size(pos, vec2(7., 7.)),
                    0.,
                    Stroke::new(1.4f32, Color32::from_rgb(0, 140, 0)),
                    StrokeKind::Inside,
                );
            } else if state.flags[15] {
                painter.circle_filled(pos, 1.8, Color32::RED);
            }
        }
    } else {
        for (circle_index, circle) in project
            .circles
            .iter()
            .enumerate()
            .filter(|(_, c)| c.visible)
        {
            let row = if circle.q.is_some() {
                3
            } else if circle.label.starts_with("VSWR") {
                6
            } else if circle.label.starts_with("Noise") {
                7
            } else if circle.label.starts_with("Gain") {
                5
            } else {
                4
            };
            let color = line_color(state, row);
            let width = state.widths[row];
            if let Some(q) = circle.q {
                for sign in [-1., 1.] {
                    let pts = (0..=400)
                        .map(|i| {
                            let r = 1000. * (i as f64 / 400.).powi(4);
                            point(gamma(c(r, sign * r * q), 1.))
                        })
                        .collect();
                    path(&painter, pts, color, width, false);
                }
            } else {
                let pts = (0..=500)
                    .map(|i| {
                        point(circle.center + C::from_polar(circle.radius, TAU * i as f64 / 500.))
                    })
                    .collect();
                path(
                    &painter,
                    pts,
                    color,
                    width,
                    !circle.highlighted && matches!(circle.family(), 'G' | 'S' | 'N'),
                );
            }
            if state.flags[7] {
                let g = if let Some(q) = circle.q {
                    gamma(c(1., q), 1.)
                } else {
                    circle.center
                        + c(
                            0.,
                            if circle.family() == 'G' {
                                -circle.radius
                            } else {
                                circle.radius
                            },
                        )
                };
                painter.text(
                    point(g) + vec2(3., 4.),
                    Align2::LEFT_TOP,
                    project.circle_name(circle_index),
                    FontId::new(18., egui::FontFamily::Name("chart".into())),
                    color,
                );
            }
        }
        if state.flags[3] {
            path(
                &painter,
                project
                    .points
                    .iter()
                    .map(|p| point(gamma(p.z, project.z0)))
                    .collect(),
                line_color(state, 12),
                state.widths[12],
                false,
            );
        }
        for (i, p) in project.points.iter().enumerate() {
            let pos = point(gamma(p.z, project.z0));
            if i == project.start {
                painter.rect_stroke(
                    Rect::from_center_size(pos, vec2(6., 6.)),
                    0.,
                    Stroke::new(2.0_f32, line_color(state, 2)),
                    StrokeKind::Inside,
                );
            } else {
                painter.circle_stroke(pos, 3., Stroke::new(1.0_f32, line_color(state, 2)));
            }
            if state.flags[5] && project.settings.labels {
                painter.text(
                    pos + vec2(5., 6.),
                    Align2::LEFT_TOP,
                    format!("DP {}", i + 1),
                    FontId::new(20., egui::FontFamily::Name("chart".into())),
                    Color32::from_rgb(0, 128, 0),
                );
            }
        }
        if let Some(start) = project.active() {
            let mut z = start.z;
            for (i, e) in project.elements.iter().enumerate() {
                path(
                    &painter,
                    (0..=200)
                        .map(|j| {
                            point(gamma(
                                e.apply_fraction(z, start.hz, j as f64 / 200.),
                                project.z0,
                            ))
                        })
                        .collect(),
                    line_color(state, 14),
                    state.widths[14],
                    false,
                );
                z = e.apply(z, start.hz);
                let pos = point(gamma(z, project.z0));
                painter.circle_stroke(pos, 3., Stroke::new(1.0_f32, Color32::BLACK));
                if state.flags[6] && project.settings.labels {
                    painter.text(
                        pos + vec2(5., 5.),
                        Align2::LEFT_TOP,
                        format!("TP {}", project.points.len() + i + 1),
                        FontId::new(20., egui::FontFamily::Name("chart".into())),
                        Color32::from_rgb(0, 128, 0),
                    );
                }
            }
        }
        let sweep = project.swept();
        path(
            &painter,
            sweep
                .iter()
                .map(|(_, z)| point(gamma(*z, project.z0)))
                .collect(),
            line_color(state, 8),
            state.widths[8],
            false,
        );
        for (i, (_, z)) in sweep.iter().enumerate() {
            let pos = point(gamma(*z, project.z0));
            if i % sweep.len().div_ceil(35).max(1) == 0 {
                painter.circle_stroke(
                    pos,
                    2.3,
                    Stroke::new(state.widths[15], line_color(state, 15)),
                );
                if state.flags[8] {
                    painter.text(
                        pos + vec2(3., 3.),
                        Align2::LEFT_TOP,
                        format!("SP {}", i + 1),
                        FontId::new(18., egui::FontFamily::Name("chart".into())),
                        Color32::from_rgb(0, 128, 0),
                    );
                }
            }
        }
    }
    let g = response.hover_pos().map(|pos| {
        c(
            ((pos.x - center.x) / r) as f64,
            ((center.y - pos.y) / r) as f64,
        ) * scale
    });
    Hit {
        response,
        g,
        radius: r,
    }
}
impl SmithApp {
    pub(in super::super) fn classic_workspace(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("classic_status")
            .exact_height(17.)
            .frame(egui::Frame::NONE.fill(FORM))
            .show(ctx, |ui| {
                ui.painter().line_segment(
                    [ui.max_rect().left_top(), ui.max_rect().right_top()],
                    Stroke::new(1.0_f32, Color32::WHITE),
                );
                ui.label(&self.status);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(DESKTOP))
            .show(ctx, |ui| {
                let bounds = ui.max_rect();
                let extent = self
                    .classic
                    .windows
                    .values()
                    .filter(|w| w.visible && !w.maximized)
                    .fold(bounds.size(), |v, w| {
                        vec2(
                            v.x.max(w.rect[0] + w.rect[2]),
                            v.y.max(w.rect[1] + if w.minimized { 29. } else { w.rect[3] }),
                        )
                    });
                let scroll = egui::ScrollArea::both()
                    .id_salt("mdi_scrolling")
                    .auto_shrink([false, false])
                    .scroll_offset(vec2(self.classic.scroll[0], self.classic.scroll[1]))
                    .show(ui, |ui| {
                        ui.set_min_size(extent);
                    });
                let desktop = scroll.inner_rect;
                self.classic.scroll = [scroll.state.offset.x, scroll.state.offset.y];
                self.classic.desktop = [
                    desktop.min.x,
                    desktop.min.y,
                    desktop.width(),
                    desktop.height(),
                ];
                let w = desktop.width();
                let h = desktop.height();
                let mut shell = ClassicState {
                    windows: std::mem::take(&mut self.classic.windows),
                    active: self.classic.active.clone(),
                    initialized: self.classic.initialized,
                    scroll: self.classic.scroll,
                    ..ClassicState::default()
                };
                if self.mode != self.classic.last_mode {
                    shell.windows.clear();
                    self.classic.last_mode = self.mode;
                    self.classic.initialized = false;
                    shell.initialized = false;
                    self.classic.layout_frames = 0;
                    shell.active = if self.mode { "s11" } else { "chart" }.into();
                }
                if !self.mode {
                    let title = if self.p.title == "Untitled" {
                        "<unnamed>".to_string()
                    } else {
                        self.path
                            .as_ref()
                            .map(|p| p.to_string_lossy().into())
                            .unwrap_or_else(|| self.p.title.clone())
                    };
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "chart",
                        &title,
                        [0., 0., w * 0.505, (w * 0.485).min(h * 0.9)],
                        Color32::WHITE,
                        |ui| self.chart_body(ui),
                    );
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "schematic",
                        "Schematic",
                        [w * 0.668, 0., w * 0.302, h * 0.225],
                        Color32::WHITE,
                        |ui| self.schematic_body(ui),
                    );
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "datapoints",
                        "Datapoints",
                        [w * 0.668, h * 0.225, w * 0.302, h * 0.225],
                        Color32::WHITE,
                        |ui| self.datapoints_body(ui),
                    );
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "circles",
                        "Circles",
                        [w * 0.668, h * 0.45, w * 0.302, h * 0.225],
                        Color32::WHITE,
                        |ui| self.circles_body(ui),
                    );
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "cursor",
                        "Cursor",
                        [w * 0.727, h * 0.675, w * 0.254, h * 0.165],
                        FORM,
                        |ui| self.cursor_body(ui),
                    );
                } else {
                    for (key, j, x, y) in [
                        ("s11", 0, 0., 0.),
                        ("s12", 2, 0.334, 0.),
                        ("s21", 1, 0., 0.5),
                        ("s22", 3, 0.334, 0.5),
                    ] {
                        let title = key.to_uppercase();
                        child(
                            ctx,
                            desktop,
                            &mut shell,
                            key,
                            &title,
                            [w * x, h * y, w * 0.333, h * 0.5],
                            Color32::WHITE,
                            |ui| self.splot_chart(ui, j),
                        );
                    }
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "source",
                        "Info Window",
                        [w * 0.668, 0., w * 0.302, h * 0.25],
                        Color32::WHITE,
                        |ui| {
                            if let Some(n) = &self.p.network {
                                egui::ScrollArea::both().show(ui, |ui| {
                                    ui.label(format!("File: {}", n.name));
                                    ui.label(format!(
                                        "{} datapoints   Z0 = {} Ω",
                                        n.points.len(),
                                        n.z0
                                    ));
                                    ui.monospace(&n.source);
                                });
                            }
                        },
                    );
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "gain_table",
                        "Gain table",
                        [w * 0.668, h * 0.25, w * 0.302, h * 0.25],
                        Color32::WHITE,
                        |ui| self.gain_table_body(ui),
                    );
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "gain",
                        "Gain graph",
                        [w * 0.668, h * 0.5, w * 0.302, h * 0.25],
                        Color32::WHITE,
                        |ui| self.graph_body(ui, true),
                    );
                    child(
                        ctx,
                        desktop,
                        &mut shell,
                        "return_loss",
                        "Return Loss graph",
                        [w * 0.668, h * 0.75, w * 0.302, h * 0.25],
                        Color32::WHITE,
                        |ui| self.graph_body(ui, false),
                    );
                }
                self.classic.windows = shell.windows;
                self.classic.active = shell.active;
                if !self.classic.initialized {
                    if self.classic.layout_size == [w, h] {
                        self.classic.layout_frames += 1;
                    } else {
                        self.classic.layout_size = [w, h];
                        self.classic.layout_frames = 0;
                    }
                    if self.classic.layout_frames >= 2 {
                        self.classic.initialized = true;
                    } else {
                        ctx.request_repaint();
                    }
                }
            });
    }
    fn chart_body(&mut self, ui: &mut Ui) {
        let hit = chart_view(ui, &self.p, &self.classic, self.zoom, None);
        self.chart_rect = Some(hit.response.rect);
        if let Some(g) = hit.g {
            self.cursor = Some(impedance(g, self.p.z0));
        }
        if hit.response.hovered() {
            let delta = ui.input(|i| i.smooth_scroll_delta.y);
            if delta != 0. {
                self.zoom = (self.zoom * (delta * 0.002).exp()).clamp(0.1, 8.);
            }
        }
        if let Some(draft) = self.pending.clone()
            && let Some(start) = self.p.active()
        {
            let hz = start.hz;
            let z = self.p.stages().last().copied().unwrap_or(start.z);
            if let Some(locus) = crate::placement::Locus::new(&draft, z, hz, self.p.z0) {
                let preview = locus.preview(
                    hit.g
                        .unwrap_or_else(|| gamma(draft.apply(z, hz), self.p.z0)),
                );
                let r = hit.response.rect;
                let center = r.center();
                let point =
                    |g: C| center + vec2(g.re as f32 * hit.radius, -g.im as f32 * hit.radius);
                path(
                    ui.painter(),
                    locus.points().into_iter().map(point).collect(),
                    line_color(&self.classic, 13),
                    self.classic.widths[13],
                    false,
                );
                let marker = point(preview.gamma);
                let p = ui.painter();
                for axis in [vec2(6., 0.), vec2(0., 6.)] {
                    p.line_segment(
                        [marker - axis, marker + axis],
                        Stroke::new(1.0_f32, Color32::BLACK),
                    );
                }
                let text = format!(
                    "{}\n{}",
                    frequency(hz),
                    crate::placement::value_label(&preview.element)
                );
                let galley = p.layout_no_wrap(text, FontId::proportional(11.), Color32::BLACK);
                let pos = marker + vec2(12., 12.);
                let pos = pos2(
                    pos.x
                        .min(r.right() - galley.size().x - 3.)
                        .max(r.left() + 3.),
                    pos.y
                        .min(r.bottom() - galley.size().y - 3.)
                        .max(r.top() + 3.),
                );
                p.galley(pos, galley, Color32::BLACK);
                self.cursor = Some(preview.z);
                self.pending = Some(preview.element.clone());
                if hit.response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                }
                if hit.response.clicked() {
                    self.checkpoint();
                    self.p.elements.push(preview.element);
                    self.pending = None;
                    self.status.clear();
                }
            }
        } else if let Some(g) = hit.g
            && hit.response.clicked()
        {
            if self.mouse && g.norm() <= 1. {
                let z = impedance(g, self.p.z0);
                self.add_point_dialog(None);
                self.point_values = [z.norm(), z.arg().to_degrees(), self.p.default_hz / 1e6];
                self.point_frequency_only = true;
                self.classic.blank_fields.clear();
                self.classic.focus_number = Some("point_frequency".into());
            } else if let Some((i, d)) = self
                .p
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| (i, (gamma(p.z, self.p.z0) - g).norm()))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                && d < 12. / hit.radius as f64
            {
                self.p.start = i;
                if hit.response.double_clicked() {
                    self.add_point_dialog(Some(i));
                }
            }
        }
        if hit.response.secondary_clicked() {
            if self.pending.take().is_none() {
                self.history.undo(&mut self.p);
            }
            self.status.clear();
        }
    }
    fn cursor_body(&self, ui: &mut Ui) {
        let width = ui.available_width();
        let col = width * 0.5;
        let input = (col - 75.).max(60.);
        let z = self.cursor;
        let g = z.map(|z| gamma(z, self.p.z0));
        let values = [
            ("Return Loss", g.map(|g| format!("{:.2} dB", -db(g.norm())))),
            ("Q", z.map(|z| format!("{:.2}", quality(z)))),
            (
                "Y",
                z.map(|z| {
                    let y = inv(z) * 1000.;
                    format!("({:.2}{:+.2}j)mS", y.re, y.im)
                }),
            ),
            ("Zo", Some(format!("{:.2}Ω", self.p.z0))),
            ("VSWR", g.map(|g| format!("{:.2} :1", vswr(g)))),
            (
                "Γ",
                g.map(|g| format!("{:.2} / {:.2}°", g.norm(), g.arg().to_degrees())),
            ),
            ("Z", z.map(|z| format!("({:.2}{:+.2}j)Ω", z.re, z.im))),
            ("Freq", self.p.active().map(|p| frequency(p.hz))),
        ];
        for (i, (name, value)) in values.iter().enumerate() {
            let column = i / 4;
            let row = i % 4;
            let x = column as f32 * col;
            let y = 8. + row as f32 * 27.;
            label(ui, x + 1., y, 65., name.to_string());
            field(ui, x + 67., y, input, value.as_deref().unwrap_or(""));
        }
    }
    fn datapoints_body(&mut self, ui: &mut Ui) {
        use table::{Cell, Event};
        let mut values: Vec<_> = self
            .p
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| (Some(i), format!("DP {}", i + 1), p.z, p.hz))
            .collect();
        if let Some(start) = self.p.active() {
            for (i, z) in self.p.stages().into_iter().enumerate().skip(1) {
                values.push((None, format!("TP {}", self.p.points.len() + i), z, start.hz));
            }
        }
        for (i, (hz, z)) in self.p.swept().into_iter().enumerate() {
            values.push((None, format!("SP {}", i + 1), z, hz));
        }
        let rows: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(row, (index, name, z, f))| {
                (
                    row,
                    vec![
                        Cell::Check(index.map(|i| i == self.p.start)),
                        name.clone().into(),
                        format!("({:.2} + j{:.2}) Ω", z.re, z.im).into(),
                        format!("Q={:.2}", quality(*z)).into(),
                        frequency(*f).into(),
                    ],
                )
            })
            .collect();
        for event in table::show(
            ui,
            "datapoints",
            &[
                ("Start DP", 49.),
                ("Point", 42.),
                ("Z", 110.),
                ("Q", 50.),
                ("Frequency", 105.),
            ],
            &rows,
            &mut self.classic.selected_datapoints,
        ) {
            match event {
                Event::Toggle(i, 0) => {
                    if let Some(index) = values[i].0 {
                        if self.p.elements.is_empty() || self.p.start == index {
                            self.p.start = index;
                        } else {
                            self.status =
                                "Start datapoint can't be changed. Elements are in design".into();
                        }
                    }
                }
                Event::DoubleClick(i) => {
                    if let Some(index) = values[i].0 {
                        self.add_point_dialog(Some(index));
                    } else if i < self.p.points.len() + self.p.elements.len() {
                        let index = i - self.p.points.len();
                        self.element(self.p.elements[index].kind, Some(index));
                    }
                }
                _ => {}
            }
        }
    }

    fn circles_body(&mut self, ui: &mut Ui) {
        use table::{Cell, Event};
        let mut indices: Vec<_> = self
            .p
            .circles
            .iter()
            .enumerate()
            .filter(|(_, c)| !matches!(c.family(), 'Q' | 'V'))
            .map(|(i, _)| i)
            .collect();
        indices.sort_by_key(|i| match self.p.circles[*i].family() {
            'G' => 0,
            'S' => 1,
            _ => 2,
        });
        if self.classic.selected_circles.is_empty()
            && let Some(i) = indices.first()
        {
            self.classic.selected_circles.push(*i);
        }
        let rows: Vec<_> = indices
            .iter()
            .map(|i| {
                let c = &self.p.circles[*i];
                (
                    *i,
                    vec![
                        Cell::Check(Some(c.visible)),
                        Cell::Check(Some(c.highlighted)),
                        c.description(&self.p.circle_name(*i)).into(),
                    ],
                )
            })
            .collect();
        for event in table::show(
            ui,
            "circles",
            &[("Visible", 44.), ("Highlighted", 66.), ("Details", 1500.)],
            &rows,
            &mut self.classic.selected_circles,
        ) {
            if let Event::Toggle(i, column) = event {
                if column == 0 {
                    self.p.circles[i].visible = !self.p.circles[i].visible;
                } else if column == 1 {
                    self.p.circles[i].highlighted = !self.p.circles[i].highlighted;
                }
            }
        }
        if ui.input(|i| i.key_pressed(Key::Delete)) {
            let selected = self.classic.selected_circles.clone();
            self.p.circles = std::mem::take(&mut self.p.circles)
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !selected.contains(i))
                .map(|(_, c)| c)
                .collect();
            self.classic.selected_circles.clear();
        }
    }

    fn schematic_body(&mut self, ui: &mut Ui) {
        let mut edit = None;
        egui::ScrollArea::both()
            .id_salt("schematic")
            .show(ui, |ui| {
                let vertical = self.p.settings.vertical;
                let count = self.p.elements.len() + usize::from(self.pending.is_some());
                let size = if vertical {
                    vec2(170., (count as f32 * 34. + 80.).max(100.))
                } else {
                    vec2((count as f32 * 34. + 70.).max(100.), 150.)
                };
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                let painter = ui.painter();
                let origin = rect.min + vec2(5., 23.);
                let map = |x: f32, y: f32| {
                    if vertical {
                        origin + vec2(y, x)
                    } else {
                        origin + vec2(x, y)
                    }
                };
                let stroke = Stroke::new(1.0_f32, Color32::BLACK);
                painter.line_segment([map(0., 0.), map((count as f32 + 1.) * 34., 0.)], stroke);
                painter.line_segment([map(0., 26.), map((count as f32 + 1.) * 34., 26.)], stroke);
                painter.rect_filled(
                    Rect::from_two_pos(map(-3., 7.), map(3., 19.)),
                    0.,
                    Color32::WHITE,
                );
                painter.rect_stroke(
                    Rect::from_two_pos(map(-3., 7.), map(3., 19.)),
                    0.,
                    stroke,
                    StrokeKind::Inside,
                );
                painter.line_segment([map(0., 0.), map(0., 7.)], stroke);
                painter.line_segment([map(0., 19.), map(0., 26.)], stroke);
                let end = (count as f32 + 1.) * 34.;
                painter.circle_stroke(map(end, 13.), 6., stroke);
                painter.line_segment([map(end, 0.), map(end, 7.)], stroke);
                painter.line_segment([map(end, 19.), map(end, 26.)], stroke);
                let wave = (0..=30)
                    .map(|i| {
                        map(
                            end - 4. + 8. * i as f32 / 30.,
                            13. - 2. * (i as f32 / 30. * TAU as f32).sin(),
                        )
                    })
                    .collect();
                painter.add(egui::Shape::line(wave, stroke));
                if self.classic.flags[10]
                    && let Some(load) = self.p.active()
                {
                    rotated_text(
                        painter,
                        map(-6., 33.),
                        format!(
                            "({:.2}{:+.2}j)Ω @{:.2}MHz",
                            load.z.re,
                            load.z.im,
                            load.hz / 1e6
                        ),
                        if vertical {
                            0.
                        } else {
                            std::f32::consts::FRAC_PI_2
                        },
                    );
                }
                for (i, e) in self
                    .p
                    .elements
                    .iter()
                    .chain(self.pending.iter())
                    .enumerate()
                {
                    let x = 34. + i as f32 * 34.;
                    let center = map(x, if e.kind.shunt() { 13. } else { 0. });
                    let r = Rect::from_center_size(
                        center,
                        if e.kind.shunt() {
                            vec2(20., 23.)
                        } else {
                            vec2(23., 20.)
                        },
                    );
                    painter.rect_filled(r.shrink(1.), 0., Color32::WHITE);
                    crate::schematic::oriented_symbol(painter, r, e, vertical);
                    if e.kind.shunt() {
                        painter.line_segment([map(x, 0.), map(x, 2.)], stroke);
                        if e.kind != Kind::OpenStub {
                            painter.line_segment([map(x, 24.), map(x, 26.)], stroke);
                        }
                    }
                    let response = ui.interact(
                        r.expand(5.)
                            .union(Rect::from_two_pos(map(x - 13., 35.), map(x + 21., 135.))),
                        ui.make_persistent_id(("component", i)),
                        Sense::click(),
                    );
                    if response.double_clicked() && i < self.p.elements.len() {
                        edit = Some(i);
                    }
                    if response.clicked()
                        && i < self.p.elements.len()
                        && self.tune
                        && !self.classic.tune_selected.contains(&i)
                    {
                        self.classic.tune_selected.push(i);
                    }
                    if self.tune && self.classic.tune_selected.contains(&i) {
                        painter.rect_stroke(
                            r.expand(3.),
                            0.,
                            Stroke::new(1.0_f32, Color32::RED),
                            StrokeKind::Inside,
                        );
                    }
                    let value = format!(
                        "{:.2}{}",
                        e.value,
                        match e.kind {
                            Kind::SeriesL | Kind::ShuntL => "nH",
                            Kind::SeriesC | Kind::ShuntC => "pF",
                            Kind::SeriesR | Kind::ShuntR => "Ω",
                            Kind::Transformer => ":1",
                            Kind::Line | Kind::OpenStub | Kind::ShortStub => "λ",
                            _ => "Ω",
                        }
                    );
                    if self.classic.flags[10] {
                        rotated_text(
                            painter,
                            map(x + 5., 33.),
                            value,
                            if vertical {
                                0.
                            } else {
                                std::f32::consts::FRAC_PI_2
                            },
                        );
                    }
                }
            });
        if let Some(i) = edit {
            self.element(self.p.elements[i].kind, Some(i));
        }
    }
    fn splot_chart(&mut self, ui: &mut Ui, j: usize) {
        let empty = formats::Network {
            name: String::new(),
            z0: self.p.z0,
            ports: 2,
            points: vec![],
            noise: vec![],
            source: String::new(),
        };
        let p = Project {
            z0: self.p.network.as_ref().map_or(self.p.z0, |n| n.z0),
            settings: self.p.settings.clone(),
            ..Project::default()
        };
        let hit = chart_view(
            ui,
            &p,
            &self.classic,
            self.zoom,
            Some((
                self.p.network.as_ref().unwrap_or(&empty),
                j,
                self.selected_s,
            )),
        );
        self.chart_rect = Some(hit.response.rect);
        if hit.response.clicked()
            && let (Some(g), Some(n)) = (hit.g, &self.p.network)
            && let Some((i, _)) = n
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| (i, (p.s[j] - g).norm()))
                .min_by(|a, b| a.1.total_cmp(&b.1))
        {
            self.selected_s = i;
        }
    }
    fn gain_table_body(&mut self, ui: &mut Ui) {
        let Some(n) = &self.p.network else {
            return;
        };
        let rows: Vec<_> = n
            .points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let st = p.stability();
                (
                    i,
                    vec![
                        format!("{:.2}", p.hz / 1e6).into(),
                        (if st.stable {
                            format!("{:.2}", st.gain_db)
                        } else {
                            "-----".into()
                        })
                        .into(),
                        (if st.stable {
                            "-----".into()
                        } else {
                            format!("{:.2}", st.gain_db)
                        })
                        .into(),
                        format!("{:.2}", st.k).into(),
                        format!("{:.2}", st.mu).into(),
                        format!("{:.2}", db(p.s[2].norm())).into(),
                    ],
                )
            })
            .collect();
        let mut selected = vec![self.selected_s];
        table::show(
            ui,
            "gain_table",
            &[
                ("Frequency [MHz]", 90.),
                ("MAG [dB]", 65.),
                ("MSG [dB]", 65.),
                ("K [-]", 55.),
                ("μ [-]", 55.),
                ("S12 [dB]", 65.),
            ],
            &rows,
            &mut selected,
        );
        if let Some(i) = selected.first() {
            self.selected_s = *i;
        }
    }
    fn graph_body(&mut self, ui: &mut Ui, gain: bool) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Button, FontId::proportional(18.));
            ui.menu_button("Diagram", |ui| {
                if ui
                    .selectable_label(self.p.settings.log_frequency, "Logarithmic")
                    .clicked()
                {
                    self.p.settings.log_frequency = true;
                    ui.close();
                }
                if ui
                    .selectable_label(!self.p.settings.log_frequency, "Linear")
                    .clicked()
                {
                    self.p.settings.log_frequency = false;
                    ui.close();
                }
                ui.separator();
                ui.checkbox(&mut self.classic.marker_info, "Show marker info");
            });
        });
        let Some(n) = &self.p.network else {
            ui.allocate_space(ui.available_size());
            return;
        };
        let (r, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        let plot = Rect::from_min_max(
            r.min + vec2(48., 12.),
            r.max - vec2(92., if self.classic.marker_info { 50. } else { 24. }),
        );
        if plot.width() < 25. || plot.height() < 25. {
            return;
        }
        let p = ui.painter();
        let transform = |f: f64| {
            if self.p.settings.log_frequency {
                f.log10()
            } else {
                f
            }
        };
        let xmin = transform(n.points[0].hz);
        let xmax = transform(n.points.last().unwrap().hz);
        let xmax = if xmax <= xmin { xmin + 1. } else { xmax };
        let max_gain = n
            .points
            .iter()
            .map(|p| p.stability().gain_db)
            .filter(|v| v.is_finite())
            .fold(10f64, f64::max);
        let ymax = if gain {
            (max_gain / 5.).ceil() * 5.
        } else {
            0.
        };
        let stability_max = n
            .points
            .iter()
            .flat_map(|p| {
                let s = p.stability();
                [s.k, s.mu]
            })
            .filter(|v| v.is_finite())
            .fold(1f64, f64::max)
            .ceil()
            .max(1.);
        let ymin = if gain {
            0.
        } else {
            (n.points
                .iter()
                .flat_map(|p| [db(p.s[0].norm()), db(p.s[3].norm())])
                .filter(|v| v.is_finite())
                .fold(-5f64, f64::min)
                / 5.)
                .floor()
                * 5.
        };
        let xy = |f: f64, v: f64| {
            pos2(
                plot.left() + ((transform(f) - xmin) / (xmax - xmin)) as f32 * plot.width(),
                plot.bottom() - ((v - ymin) / (ymax - ymin)) as f32 * plot.height(),
            )
        };
        for i in 0..=5 {
            let y = plot.bottom() - plot.height() * i as f32 / 5.;
            p.line_segment(
                [pos2(plot.left(), y), pos2(plot.right(), y)],
                Stroke::new(
                    self.classic.widths[if gain { 23 } else { 24 }],
                    line_color(&self.classic, if gain { 23 } else { 24 }),
                ),
            );
            p.text(
                pos2(plot.left() - 4., y),
                Align2::RIGHT_CENTER,
                format!("{:.2}", ymin + (ymax - ymin) * i as f64 / 5.),
                FontId::proportional(9.),
                Color32::BLACK,
            );
            if gain {
                p.text(
                    pos2(plot.right() + 4., y),
                    Align2::LEFT_CENTER,
                    format!("{:.2}", stability_max * i as f64 / 5.),
                    FontId::proportional(9.),
                    Color32::BLACK,
                );
            }
        }
        for i in 0..=5 {
            let x = plot.left() + plot.width() * i as f32 / 5.;
            p.line_segment(
                [pos2(x, plot.top()), pos2(x, plot.bottom())],
                Stroke::new(
                    self.classic.widths[if gain { 23 } else { 24 }] * 0.7,
                    line_color(&self.classic, if gain { 23 } else { 24 }),
                ),
            );
            let f = if self.p.settings.log_frequency {
                10f64.powf(xmin + (xmax - xmin) * i as f64 / 5.)
            } else {
                xmin + (xmax - xmin) * i as f64 / 5.
            };
            p.text(
                pos2(x, plot.bottom() + 9.),
                Align2::CENTER_CENTER,
                format!("{:.2}", f / 1e6),
                FontId::proportional(9.),
                Color32::BLACK,
            );
        }
        p.text(
            pos2(plot.center().x, plot.bottom() + 23.),
            Align2::CENTER_CENTER,
            "f [MHz]",
            FontId::proportional(10.),
            Color32::BLACK,
        );
        p.text(
            plot.left_top() - vec2(29., -plot.height() * 0.5),
            Align2::CENTER_CENTER,
            "[dB]",
            FontId::proportional(9.),
            Color32::BLACK,
        );
        let selected = self.selected_s.min(n.points.len() - 1);
        let mut curves: Vec<GraphCurve<'_>> = Vec::new();
        if gain {
            curves.push((
                "MAG",
                17,
                n.points
                    .iter()
                    .filter(|p| p.stability().stable)
                    .map(|p| (p.hz, p.stability().gain_db))
                    .collect(),
            ));
            curves.push((
                "MSG",
                18,
                n.points
                    .iter()
                    .filter(|p| !p.stability().stable)
                    .map(|p| (p.hz, p.stability().gain_db))
                    .collect(),
            ));
            curves.push((
                "K",
                19,
                n.points
                    .iter()
                    .map(|p| (p.hz, p.stability().k / stability_max * ymax))
                    .collect(),
            ));
            curves.push((
                "μ",
                20,
                n.points
                    .iter()
                    .map(|p| (p.hz, p.stability().mu / stability_max * ymax))
                    .collect(),
            ));
        } else {
            curves.push((
                "Return Loss S11",
                21,
                n.points.iter().map(|p| (p.hz, db(p.s[0].norm()))).collect(),
            ));
            if n.ports == 2 {
                curves.push((
                    "Return Loss S22",
                    22,
                    n.points.iter().map(|p| (p.hz, db(p.s[3].norm()))).collect(),
                ));
            }
        }
        let clipped = p.with_clip_rect(plot.expand(1.));
        for (i, (name, row, points)) in curves.iter().enumerate() {
            path(
                &clipped,
                points
                    .iter()
                    .filter(|(_, v)| v.is_finite())
                    .map(|(f, v)| xy(*f, *v))
                    .collect(),
                line_color(&self.classic, *row),
                self.classic.widths[*row],
                false,
            );
            let y = plot.top() + 9. + i as f32 * 13.;
            let legend_x = plot.right() + if gain { 28. } else { 4. };
            p.line_segment(
                [pos2(legend_x, y), pos2(legend_x + 14., y)],
                Stroke::new(1.5f32, line_color(&self.classic, *row)),
            );
            p.text(
                pos2(legend_x + 17., y),
                Align2::LEFT_CENTER,
                *name,
                FontId::proportional(if gain { 9. } else { 8. }),
                Color32::BLACK,
            );
            let sp = &n.points[selected];
            let st = sp.stability();
            let value = match *row {
                17 if st.stable => Some(st.gain_db),
                18 if !st.stable => Some(st.gain_db),
                19 => Some(st.k / stability_max * ymax),
                20 => Some(st.mu / stability_max * ymax),
                21 => Some(db(sp.s[0].norm())),
                22 => Some(db(sp.s[3].norm())),
                _ => None,
            };
            if let Some(v) = value
                && v.is_finite()
            {
                clipped.rect_filled(
                    Rect::from_center_size(xy(sp.hz, v), vec2(5., 5.)),
                    0.,
                    line_color(&self.classic, *row),
                );
            }
        }
        if self.classic.marker_info {
            let sp = &n.points[selected];
            let st = sp.stability();
            let text = if gain {
                format!(
                    "Cursor position     MAG {}    MSG {}    K {:.2}    μ {:.2}",
                    if st.stable {
                        format!("{:.2}", st.gain_db)
                    } else {
                        "-".into()
                    },
                    if !st.stable {
                        format!("{:.2}", st.gain_db)
                    } else {
                        "-".into()
                    },
                    st.k,
                    st.mu
                )
            } else {
                format!(
                    "Cursor position     RL S11 {:.2}    RL S22 {:.2}",
                    db(sp.s[0].norm()),
                    db(sp.s[3].norm())
                )
            };
            p.text(
                pos2(r.left() + 9., r.bottom() - 12.),
                Align2::LEFT_CENTER,
                text,
                FontId::proportional(9.),
                Color32::BLACK,
            );
        }
        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let x = (pos.x - plot.left()) / plot.width();
            let v = xmin + (xmax - xmin) * x as f64;
            if let Some((i, _)) = n
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| (i, (transform(p.hz) - v).abs()))
                .min_by(|a, b| a.1.total_cmp(&b.1))
            {
                self.selected_s = i;
            }
        }
    }
}
