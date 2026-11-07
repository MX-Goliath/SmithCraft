//! Reproduction of the Smith V4.1 MDI shell. Layout and drawing are deliberately
//! kept separate from the RF model so a later redesign does not affect calculations.
use super::*;
use egui::{Align2, FontId, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
mod dialogs;
mod menus;
mod native;
mod table;
mod workspace;

pub(super) const FORM: Color32 = Color32::from_rgb(240, 240, 240);
pub(super) const DESKTOP: Color32 = Color32::from_rgb(171, 171, 171);
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct ChildState {
    pub rect: [f32; 4],
    pub minimized: bool,
    pub maximized: bool,
    pub visible: bool,
    #[serde(default)]
    pub customized: bool,
}
impl ChildState {
    fn new(r: [f32; 4]) -> Self {
        Self {
            rect: r,
            minimized: false,
            maximized: false,
            visible: true,
            customized: false,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct ClassicState {
    #[serde(default)]
    pub version: u32,
    pub windows: BTreeMap<String, ChildState>,
    pub active: String,
    pub initialized: bool,
    #[serde(skip)]
    pub layout_size: [f32; 2],
    #[serde(skip)]
    pub layout_frames: u8,
    pub scroll: [f32; 2],
    pub last_mode: bool,
    pub default_splot: bool,
    pub recent: Vec<PathBuf>,
    pub toolbar_positions: BTreeMap<String, [f32; 2]>,
    pub settings_tab: usize,
    pub circle_tab: usize,
    pub point_unit: usize,
    pub settings_z0_unit: usize,
    pub settings_frequency_unit: usize,
    pub r_unit: usize,
    pub l_unit: usize,
    pub c_unit: usize,
    pub circle_formats: [usize; 4],
    pub noise_polar: bool,
    pub noise_normal: bool,
    pub marker_info: bool,
    pub copy_dialog: bool,
    pub print_dialog: bool,
    pub print_preview: bool,
    pub preview_open: bool,
    pub preview_zoom: usize,
    pub preview_page: usize,
    pub parameter_dialog: bool,
    pub about: bool,
    pub copy_item: usize,
    pub print_items: [bool; 22],
    pub export_dialog: bool,
    pub export_circles: bool,
    pub delimiter: usize,
    pub q_defined: [bool; 6],
    pub vswr_defined: [bool; 6],
    pub q_other: String,
    pub vswr_other: String,
    pub tune_selected: Vec<usize>,
    pub selected_circle: Option<usize>,
    pub selected_point: Option<usize>,
    pub selected_datapoints: Vec<usize>,
    pub selected_circles: Vec<usize>,
    pub color_rows: Vec<[u8; 3]>,
    pub widths: Vec<f32>,
    pub flags: Vec<bool>,
    pub style_project: Option<Project>,
    pub dialog_positions: BTreeMap<String, [f32; 2]>,
    pub fields: BTreeMap<String, String>,
    pub blank_fields: std::collections::BTreeSet<String>,
    pub focus_number: Option<String>,
    pub custom_colors: [u32; 16],
    pub sweep_units: [usize; 3],
    pub import_for_circle: bool,
    pub capture_pending: Option<String>,
    pub desktop: [f32; 4],
    pub help_topic: usize,
    pub circle_frequency: Option<f64>,
    pub circle_backup: Option<Vec<Circle>>,
    pub element_backup: Option<Project>,
    pub sweep_backup: Option<Sweep>,
    pub style_backup: Option<Box<ClassicState>>,
}
impl Default for ClassicState {
    fn default() -> Self {
        Self {
            version: 3,
            windows: BTreeMap::new(),
            active: "chart".into(),
            initialized: false,
            layout_size: [0.; 2],
            layout_frames: 0,
            scroll: [0.; 2],
            last_mode: false,
            default_splot: false,
            recent: vec![],
            toolbar_positions: BTreeMap::new(),
            settings_tab: 0,
            circle_tab: 0,
            point_unit: 2,
            settings_z0_unit: 3,
            settings_frequency_unit: 1,
            r_unit: 1,
            l_unit: 1,
            c_unit: 0,
            circle_formats: [0; 4],
            noise_polar: true,
            noise_normal: true,
            marker_info: true,
            copy_dialog: false,
            print_dialog: false,
            print_preview: false,
            preview_open: false,
            preview_zoom: 0,
            preview_page: 0,
            parameter_dialog: false,
            about: false,
            copy_item: 0,
            print_items: [true; 22],
            export_dialog: false,
            export_circles: false,
            delimiter: 1,
            q_defined: [false; 6],
            vswr_defined: [false; 6],
            q_other: String::new(),
            vswr_other: String::new(),
            tune_selected: vec![],
            selected_circle: None,
            selected_point: None,
            selected_datapoints: vec![0],
            selected_circles: vec![],
            color_rows: vec![
                [255, 0, 0],
                [0, 128, 255],
                [0, 0, 0],
                [0, 128, 128],
                [128, 0, 128],
                [0, 192, 0],
                [128, 64, 0],
                [0, 128, 128],
                [0, 192, 0],
                [0, 0, 0],
                [255, 0, 0],
                [0, 128, 255],
                [255, 0, 0],
                [255, 128, 0],
                [64, 0, 64],
                [255, 128, 0],
                [0, 0, 255],
                [255, 0, 0],
                [0, 0, 255],
                [0, 128, 0],
                [255, 128, 64],
                [0, 0, 255],
                [255, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ],
            widths: vec![
                1., 1., 2., 2., 2., 1., 2., 1., 1., 2., 1., 1., 1., 1., 2., 2., 1., 2., 2., 2., 2.,
                2., 2., 1., 1.,
            ],
            flags: vec![
                false, true, true, false, true, true, true, true, false, false, true, false, true,
                true, true, true, true, true, true,
            ],
            style_project: None,
            dialog_positions: BTreeMap::new(),
            fields: BTreeMap::new(),
            blank_fields: std::collections::BTreeSet::new(),
            focus_number: None,
            custom_colors: [0xffffff; 16],
            sweep_units: [2; 3],
            import_for_circle: false,
            capture_pending: None,
            desktop: [0.; 4],
            help_topic: 0,
            circle_frequency: None,
            circle_backup: None,
            element_backup: None,
            sweep_backup: None,
            style_backup: None,
        }
    }
}
impl ClassicState {
    pub fn upgrade(&mut self) {
        if self.version < 3 {
            for (index, old, new) in [
                (1, [0, 170, 255], [0, 128, 255]),
                (5, [0, 180, 0], [0, 192, 0]),
                (6, [160, 90, 0], [128, 64, 0]),
                (8, [0, 200, 0], [0, 192, 0]),
                (11, [0, 170, 255], [0, 128, 255]),
                (14, [128, 0, 128], [64, 0, 64]),
                (19, [0, 150, 0], [0, 128, 0]),
                (20, [255, 128, 0], [255, 128, 64]),
            ] {
                if self.color_rows.get(index) == Some(&old) {
                    self.color_rows[index] = new;
                }
            }
            if self.flags.len() > 8 {
                self.flags[8] = false;
            }
            self.version = 3;
        }
    }
    pub fn reset_transient(&mut self) {
        self.initialized = !self.windows.is_empty();
        self.copy_dialog = false;
        self.print_dialog = false;
        self.preview_open = false;
        self.parameter_dialog = false;
        self.about = false;
        self.export_dialog = false;
        self.style_project = None;
        self.style_backup = None;
        self.circle_backup = None;
        self.element_backup = None;
        self.sweep_backup = None;
        self.capture_pending = None;
        self.import_for_circle = false;
        self.fields.clear();
        self.blank_fields.clear();
        self.focus_number = None;
        self.help_topic = 0;
    }
}
pub(super) fn setup(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\tahoma.ttf") {
        fonts
            .font_data
            .insert("Tahoma".into(), egui::FontData::from_owned(bytes).into());
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "Tahoma".into());
    }
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\arial.ttf") {
        fonts
            .font_data
            .insert("Arial".into(), egui::FontData::from_owned(bytes).into());
        fonts.families.insert(
            egui::FontFamily::Name("chart".into()),
            vec!["Arial".into(), "Tahoma".into()],
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "Arial".into());
    }
    // Linux and portable Windows installations may not have Arial/Tahoma.
    // The chart family must always exist, including on a clean AppImage host.
    let fallback = fonts.families[&egui::FontFamily::Proportional].clone();
    fonts
        .families
        .entry(egui::FontFamily::Name("chart".into()))
        .or_insert(fallback);
    ctx.set_fonts(fonts);
    ctx.set_visuals(egui::Visuals::light());
    ctx.set_theme(egui::Theme::Light);
    ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
    let mut style = (*ctx.style()).clone();
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(11.));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(11.));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(10.));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, FontId::proportional(11.));
    style.spacing.item_spacing = vec2(4., 4.);
    style.spacing.button_padding = vec2(5., 1.);
    style.spacing.interact_size = vec2(16., 17.);
    style.spacing.icon_width = 11.;
    style.spacing.icon_spacing = 3.;
    style.spacing.menu_margin = egui::Margin::same(2);
    style.visuals.window_fill = FORM;
    style.visuals.panel_fill = FORM;
    style.visuals.window_corner_radius = 0.into();
    style.visuals.menu_corner_radius = 0.into();
    style.visuals.widgets.inactive.corner_radius = 0.into();
    style.visuals.widgets.hovered.corner_radius = 0.into();
    style.visuals.widgets.active.corner_radius = 0.into();
    style.visuals.selection.bg_fill = Color32::from_rgb(51, 153, 255);
    style.visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
    style.visuals.override_text_color = Some(Color32::BLACK);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(235, 235, 235);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_gray(160));
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(229, 241, 251);
    ctx.set_style(style);
}
pub(super) fn gradient(p: &egui::Painter, r: Rect, a: Color32, b: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(r.left_top(), a);
    mesh.colored_vertex(r.right_top(), a);
    mesh.colored_vertex(r.right_bottom(), b);
    mesh.colored_vertex(r.left_bottom(), b);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(egui::Shape::mesh(mesh));
}
pub(super) fn at(ui: &Ui, x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(ui.max_rect().min + vec2(x, y), vec2(w, h))
}
pub(super) fn enabled_at(ui: &mut Ui, enabled: bool, content: impl FnOnce(&mut Ui)) {
    let bounds = ui.max_rect();
    ui.scope_builder(UiBuilder::new().max_rect(bounds), |ui| {
        if !enabled {
            ui.disable();
        }
        content(ui);
    });
}
pub(super) fn label(ui: &mut Ui, x: f32, y: f32, w: f32, text: impl Into<String>) {
    let r = at(ui, x, y, w, 17.);
    ui.painter()
        .with_clip_rect(r.intersect(ui.clip_rect()))
        .text(
            r.left_center(),
            Align2::LEFT_CENTER,
            text.into(),
            FontId::proportional(11.),
            ui.visuals().text_color(),
        );
}
pub(super) fn group(ui: &mut Ui, x: f32, y: f32, w: f32, h: f32, title: &str) {
    let r = at(ui, x, y, w, h);
    let p = ui.painter();
    p.rect_stroke(
        r.translate(vec2(1., 1.)),
        0.,
        Stroke::new(1.0_f32, Color32::WHITE),
        StrokeKind::Inside,
    );
    p.rect_stroke(
        r,
        0.,
        Stroke::new(1.0_f32, Color32::from_gray(180)),
        StrokeKind::Inside,
    );
    if !title.is_empty() {
        let galley = p.layout_no_wrap(title.into(), FontId::proportional(9.), Color32::BLACK);
        let pos = r.min + vec2(6., -6.);
        p.rect_filled(
            Rect::from_min_size(pos - vec2(2., 0.), galley.size() + vec2(4., 0.)),
            0.,
            FORM,
        );
        p.galley(pos, galley, Color32::BLACK);
    }
}
pub(super) fn button(ui: &mut Ui, x: f32, y: f32, w: f32, h: f32, text: &str) -> bool {
    ui.put(at(ui, x, y, w, h), egui::Button::new(text))
        .clicked()
}
pub(super) fn green_button(ui: &mut Ui, x: f32, y: f32, w: f32, h: f32, text: &str) -> bool {
    ui.put(
        at(ui, x, y, w, h),
        egui::Button::new(text).fill(Color32::from_rgb(144, 238, 144)),
    )
    .clicked()
}
pub(super) fn field(ui: &mut Ui, x: f32, y: f32, w: f32, text: &str) {
    let r = at(ui, x, y, w, 15.);
    ui.painter().rect_filled(r, 0., FORM);
    ui.painter().line_segment(
        [r.left_bottom(), r.left_top()],
        Stroke::new(1.0_f32, Color32::from_gray(145)),
    );
    ui.painter().line_segment(
        [r.left_top(), r.right_top()],
        Stroke::new(1.0_f32, Color32::from_gray(145)),
    );
    ui.painter().line_segment(
        [r.right_top(), r.right_bottom()],
        Stroke::new(1.0_f32, Color32::WHITE),
    );
    ui.painter().line_segment(
        [r.right_bottom(), r.left_bottom()],
        Stroke::new(1.0_f32, Color32::WHITE),
    );
    ui.painter().text(
        r.left_center() + vec2(3., 0.),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(9.),
        Color32::BLACK,
    );
}
pub(super) fn number(
    ui: &mut Ui,
    state: &mut ClassicState,
    id: &str,
    mut r: Rect,
    value: &mut f64,
    enabled: bool,
) -> bool {
    let blank_key = id;
    let compact = ["point_", "settings_", "sweep_"]
        .iter()
        .any(|p| id.starts_with(p));
    if compact {
        r.set_height(14.);
    }
    let id = ui.make_persistent_id(id);
    let focused = ui.memory(|m| m.has_focus(id));
    let key = format!("{:?}", id);
    let mut text = if focused {
        state.fields.get(&key).cloned().unwrap_or_else(|| {
            if state.blank_fields.contains(blank_key) {
                String::new()
            } else {
                format_number(*value)
            }
        })
    } else {
        if state.blank_fields.contains(blank_key) {
            String::new()
        } else {
            format_number(*value)
        }
    };
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            ui.put(
                r,
                egui::TextEdit::singleline(&mut text)
                    .id(id)
                    .font(FontId::proportional(if compact { 9. } else { 10. }))
                    .margin(vec2(2., 1.)),
            )
        })
        .inner;
    if state.focus_number.as_deref() == Some(blank_key) {
        response.request_focus();
        state.focus_number = None;
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            0.,
            Stroke::new(1f32, Color32::from_rgb(70, 140, 70)),
            StrokeKind::Inside,
        );
    }
    let changed = response.changed();
    if changed {
        if text.trim().is_empty() {
            state.blank_fields.insert(blank_key.into());
        } else {
            state.blank_fields.remove(blank_key);
        }
    }
    if changed
        && let Ok(v) = text.replace(',', ".").parse::<f64>()
        && v.is_finite()
    {
        *value = v;
    }
    state.fields.insert(key, text);
    changed
}
fn format_number(v: f64) -> String {
    if v == 0. {
        "0.00".into()
    } else if v.abs() >= 1e9 || v.abs() < 0.01 {
        format!("{v:.2e}")
    } else {
        format!("{v:.2}")
    }
}
pub(super) fn radio(ui: &mut Ui, x: f32, y: f32, w: f32, on: bool, text: &str) -> bool {
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(at(ui, x, y, w, 17.))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
            ui.add(egui::RadioButton::new(on, text)).clicked()
        },
    )
    .inner
}
pub(super) fn checkbox(ui: &mut Ui, x: f32, y: f32, w: f32, value: &mut bool, text: &str) -> bool {
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(at(ui, x, y, w, 17.))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
            ui.add(egui::Checkbox::new(value, text)).changed()
        },
    )
    .inner
}
pub(super) fn combo(
    ui: &mut Ui,
    id: &str,
    x: f32,
    y: f32,
    w: f32,
    value: &mut usize,
    items: &[&str],
) -> bool {
    let before = *value;
    ui.scope_builder(UiBuilder::new().max_rect(at(ui, x, y, w, 18.)), |ui| {
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(9.));
        ui.style_mut()
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(9.));
        ui.spacing_mut().interact_size.y = 14.;
        ui.spacing_mut().button_padding.y = 0.;
        egui::ComboBox::from_id_salt(id)
            .selected_text(items[(*value).min(items.len() - 1)])
            .width(w)
            .height(180.)
            .show_ui(ui, |ui| {
                for (i, text) in items.iter().enumerate() {
                    ui.selectable_value(value, i, *text);
                }
            });
    });
    before != *value
}

/// MDI child with the same border, gradient caption, and window controls as
/// the original. Content remains a real egui work surface, not a screenshot.
#[allow(clippy::too_many_arguments)]
pub(super) fn child<R>(
    ctx: &egui::Context,
    desktop: Rect,
    state: &mut ClassicState,
    key: &str,
    title: &str,
    default: [f32; 4],
    fill: Color32,
    content: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let mut window = state
        .windows
        .get(key)
        .cloned()
        .unwrap_or_else(|| ChildState::new(default));
    if !window.customized && !state.initialized {
        window.rect = default;
    }
    if !window.visible {
        return None;
    }
    let mut r = if window.maximized {
        desktop
    } else {
        Rect::from_min_size(
            desktop.min
                + vec2(
                    window.rect[0] - state.scroll[0],
                    window.rect[1] - state.scroll[1],
                ),
            vec2(window.rect[2], window.rect[3]),
        )
    };
    if window.minimized {
        r.set_height(29.);
    }
    let active = state.active == key;
    let id = Id::new(("mdi", key));
    let mut output = None;
    egui::Area::new(id)
        .fixed_pos(r.min)
        .movable(false)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.set_min_size(r.size());
            ui.set_max_size(r.size());
            let r = Rect::from_min_size(ui.min_rect().min, r.size());
            ui.set_clip_rect(desktop);
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos())
                && r.contains(pos)
                && ui.input(|i| i.pointer.any_pressed())
                && ctx.layer_id_at(pos) == Some(ui.layer_id())
            {
                state.active = key.into();
                ctx.move_to_top(ui.layer_id());
            }
            let p = ui.painter();
            p.rect_filled(r, 0., Color32::from_rgb(200, 220, 235));
            p.rect_stroke(
                r,
                0.,
                Stroke::new(1.0_f32, Color32::from_rgb(92, 111, 126)),
                StrokeKind::Inside,
            );
            p.rect_stroke(
                r.shrink(1.),
                0.,
                Stroke::new(1.0_f32, Color32::from_rgb(243, 250, 255)),
                StrokeKind::Inside,
            );
            let caption =
                Rect::from_min_max(r.min + vec2(3., 3.), pos2(r.right() - 3., r.top() + 28.));
            gradient(
                p,
                caption,
                if active {
                    Color32::from_rgb(145, 177, 205)
                } else {
                    Color32::from_rgb(191, 208, 222)
                },
                Color32::from_rgb(226, 235, 243),
            );
            p.text(
                caption.left_center() + vec2(5., 0.),
                Align2::LEFT_CENTER,
                title,
                FontId::proportional(10.),
                Color32::from_gray(if active { 65 } else { 135 }),
            );
            let drag = ui.interact(
                Rect::from_min_max(caption.min, pos2(caption.right() - 102., caption.bottom())),
                id.with("title"),
                Sense::click_and_drag(),
            );
            if drag.clicked() || drag.dragged() {
                state.active = key.into();
                ctx.move_to_top(ui.layer_id());
            }
            if drag.dragged() && !window.maximized {
                window.customized = true;
                let d = drag.drag_delta();
                window.rect[0] += d.x;
                window.rect[1] += d.y;
            }
            if drag.double_clicked() {
                window.maximized = !window.maximized;
                window.minimized = false;
            }
            for i in 0..if key == "cursor" { 0 } else { 3 } {
                let cr = Rect::from_min_size(
                    pos2(r.right() - 111. + i as f32 * 34., r.top() + 8.),
                    vec2(32., 19.),
                );
                let response = ui.interact(
                    cr,
                    id.with(i),
                    if i == 2 {
                        Sense::hover()
                    } else {
                        Sense::click()
                    },
                );
                let p = ui.painter();
                gradient(
                    p,
                    cr,
                    Color32::from_rgb(237, 245, 250),
                    Color32::from_rgb(173, 198, 220),
                );
                p.rect_stroke(
                    cr,
                    0.,
                    Stroke::new(1.0_f32, Color32::from_rgb(123, 145, 165)),
                    StrokeKind::Inside,
                );
                let col = if i == 2 {
                    Color32::from_gray(165)
                } else {
                    Color32::from_rgb(80, 105, 125)
                };
                let c = cr.center();
                match i {
                    0 => {
                        p.line_segment(
                            [c + vec2(-4., 3.), c + vec2(4., 3.)],
                            Stroke::new(1.0_f32, col),
                        );
                    }
                    1 => {
                        p.rect_stroke(
                            Rect::from_center_size(c, vec2(8., 7.)),
                            0.,
                            Stroke::new(1.0_f32, col),
                            StrokeKind::Inside,
                        );
                    }
                    _ => {
                        p.line_segment(
                            [c + vec2(-4., -4.), c + vec2(4., 4.)],
                            Stroke::new(1.0_f32, col),
                        );
                        p.line_segment(
                            [c + vec2(-4., 4.), c + vec2(4., -4.)],
                            Stroke::new(1.0_f32, col),
                        );
                    }
                }
                if response.clicked() {
                    state.active = key.into();
                    ctx.move_to_top(ui.layer_id());
                    match i {
                        0 => window.minimized = !window.minimized,
                        1 => {
                            window.maximized = !window.maximized;
                            window.minimized = false;
                        }
                        _ => window.visible = false,
                    }
                }
            }
            if !window.minimized {
                let body = Rect::from_min_max(r.min + vec2(7., 29.), r.max - vec2(7., 7.));
                ui.painter().rect_filled(body, 0., fill);
                ui.scope_builder(
                    UiBuilder::new()
                        .max_rect(body)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        ui.set_clip_rect(body.intersect(desktop));
                        output = Some(content(ui));
                    },
                );
                let resize = Rect::from_min_size(r.max - vec2(10., 10.), vec2(10., 10.));
                let response = ui.interact(resize, id.with("resize"), Sense::drag());
                if response.dragged() && !window.maximized {
                    window.customized = true;
                    let delta = response.drag_delta();
                    window.rect[2] = (window.rect[2] + delta.x).max(160.);
                    window.rect[3] = (window.rect[3] + delta.y).max(65.);
                }
            }
        });
    state.windows.insert(key.into(), window);
    output
}
pub(super) fn dialog<R>(
    ctx: &egui::Context,
    state: &mut ClassicState,
    key: &str,
    title: &str,
    size: Vec2,
    close_box: bool,
    mut content: impl FnMut(&mut Ui) -> R,
) -> (Option<R>, bool) {
    let position = state.dialog_positions.get(key).copied().unwrap_or([0., 0.]);
    let viewport = egui::ViewportId::from_hash_of(("smith_dialog", key));
    let mut output = None;
    let mut close = false;
    let mut moved = None;
    let builder = egui::ViewportBuilder::default()
        .with_title(title)
        .with_inner_size(size)
        .with_position(position)
        .with_decorations(false)
        .with_resizable(false)
        .with_taskbar(false);
    ctx.show_viewport_immediate(viewport, builder, |child_ctx, _| {
        child_ctx.set_theme(egui::Theme::Light);
        if let Some(rect) = child_ctx.input(|i| i.viewport().outer_rect) {
            moved = Some([rect.min.x, rect.min.y]);
        }
        close = child_ctx.input(|i| i.viewport().close_requested() || i.key_pressed(Key::Escape));
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(FORM))
            .show(child_ctx, |ui| {
                let r = ui.max_rect();
                ui.painter().rect_filled(r, 6., FORM);
                ui.painter().rect_stroke(
                    r,
                    6.,
                    Stroke::new(1.0_f32, Color32::from_gray(160)),
                    StrokeKind::Inside,
                );
                let caption = Rect::from_min_size(r.min + vec2(1., 1.), vec2(r.width() - 2., 28.));
                gradient(
                    ui.painter(),
                    caption,
                    Color32::from_rgb(246, 247, 252),
                    Color32::from_rgb(234, 238, 245),
                );
                ui.painter().text(
                    caption.left_center() + vec2(8., 0.),
                    Align2::LEFT_CENTER,
                    title,
                    FontId::proportional(11.),
                    Color32::BLACK,
                );
                let drag = ui.interact(
                    Rect::from_min_max(caption.min, caption.max - vec2(30., 0.)),
                    Id::new((key, "caption")),
                    Sense::drag(),
                );
                if drag.drag_started() {
                    child_ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if close_box {
                    let cr = Rect::from_min_size(
                        pos2(caption.right() - 28., caption.top()),
                        vec2(26., 26.),
                    );
                    let response = ui.interact(cr, Id::new((key, "close")), Sense::click());
                    ui.painter().text(
                        cr.center(),
                        Align2::CENTER_CENTER,
                        "×",
                        FontId::proportional(16.),
                        Color32::from_gray(100),
                    );
                    close |= response.clicked();
                }
                let body = Rect::from_min_max(r.min + vec2(5., 32.), r.max - vec2(5., 5.));
                ui.scope_builder(
                    UiBuilder::new()
                        .max_rect(body)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        ui.set_clip_rect(body);
                        output = Some(content(ui));
                    },
                );
            });
    });
    if let Some(position) = moved {
        state.dialog_positions.insert(key.into(), position);
    }
    (output, close)
}
