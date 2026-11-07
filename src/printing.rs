//! Shared vector pages for the native print preview and Windows printer.
use crate::{chart, formats, project::*, rf::*};
use eframe::egui::{self, Color32, FontId, Rect, Stroke, vec2};
use std::f64::consts::TAU;

#[derive(Clone)]
pub enum Mark {
    Path(Vec<[f32; 2]>, [u8; 3], f32),
    Text([f32; 2], String, f32, [u8; 3]),
    Dot([f32; 2], [u8; 3]),
}
pub struct Page {
    pub title: String,
    pub marks: Vec<Mark>,
}
impl Page {
    fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            marks: vec![Mark::Text([30., 25.], title.clone(), 14., [0; 3])],
            title,
        }
    }
    fn scene(&mut self, scene: Vec<chart::Primitive>) {
        let xy = |p: [f64; 2]| [297.5 + p[0] as f32 * 245., 385. - p[1] as f32 * 245.];
        for item in scene {
            self.marks.push(match item {
                chart::Primitive::Path(p, c, w) => Mark::Path(
                    p.into_iter()
                        .filter(|p| p.iter().all(|x| x.is_finite()))
                        .map(xy)
                        .collect(),
                    c,
                    w * 0.6,
                ),
                chart::Primitive::Dot(p, c, _) => Mark::Dot(xy(p), c),
                chart::Primitive::Text(p, t, c) => Mark::Text(xy(p), t, 8., c),
            });
        }
    }
}
fn table(out: &mut Vec<Page>, title: &str, lines: impl IntoIterator<Item = String>) {
    let lines: Vec<_> = lines.into_iter().collect();
    for chunk in lines.chunks(48) {
        let mut page = Page::new(title);
        for (i, line) in chunk.iter().enumerate() {
            page.marks.push(Mark::Text(
                [30., 60. + i as f32 * 15.],
                line.clone(),
                9.,
                [0; 3],
            ));
        }
        out.push(page);
    }
}
pub fn pages(p: &Project, splot: bool, selected: [bool; 22]) -> Vec<Page> {
    let mut out = vec![];
    if splot {
        let Some(n) = &p.network else {
            return out;
        };
        if n.points.is_empty() {
            return out;
        }
        for (slot, j, name) in [
            (14, 0, "S11"),
            (15, 2, "S12"),
            (16, 1, "S21"),
            (17, 3, "S22"),
        ] {
            if !selected[slot] || (n.ports == 1 && j != 0) {
                continue;
            }
            let mut page = Page::new(format!("{} — {name}", n.name));
            let polar = j == 1 || j == 2;
            let scale = if polar {
                n.points.iter().map(|p| p.s[j].norm()).fold(1e-20, f64::max) * 1.1
            } else {
                1.
            };
            let scene = if polar {
                (1..=4)
                    .map(|k| {
                        chart::Primitive::Path(
                            (0..=240)
                                .map(|i| {
                                    let a = TAU * i as f64 / 240.;
                                    [a.cos() * k as f64 / 4., a.sin() * k as f64 / 4.]
                                })
                                .collect(),
                            [0, 0, 255],
                            0.6,
                        )
                    })
                    .collect()
            } else {
                chart::grid(&Project {
                    z0: n.z0,
                    settings: p.settings.clone(),
                    ..Project::default()
                })
            };
            page.scene(scene);
            page.scene(vec![chart::Primitive::Path(
                n.points
                    .iter()
                    .map(|p| [p.s[j].re / scale, p.s[j].im / scale])
                    .collect(),
                [255, 0, 0],
                1.5,
            )]);
            out.push(page);
        }
        if selected[18] {
            table(
                &mut out,
                "Info Window",
                n.source.lines().map(str::to_string),
            );
        }
        if selected[19] {
            table(
                &mut out,
                "Gain table",
                std::iter::once(
                    "MHz                  MAG/MSG [dB]           K                   μ".into(),
                )
                .chain(n.points.iter().map(|p| {
                    let s = p.stability();
                    format!(
                        "{:<22.2}{:<24.2}{:<20.2}{:.2}",
                        p.hz / 1e6,
                        s.gain_db,
                        s.k,
                        s.mu
                    )
                })),
            );
        }
        for (slot, gain, title) in [(20, true, "Gain graph"), (21, false, "Return Loss graph")] {
            if !selected[slot] || (gain && n.ports == 1) {
                continue;
            }
            let mut page = Page::new(title);
            let tx = |f: f64| {
                if p.settings.log_frequency {
                    f.log10()
                } else {
                    f
                }
            };
            let a = tx(n.points[0].hz);
            let b = tx(n.points.last().unwrap().hz);
            let top = if gain {
                n.points
                    .iter()
                    .map(|p| p.stability().gain_db)
                    .filter(|v| v.is_finite())
                    .fold(10., f64::max)
                    .ceil()
            } else {
                0.
            };
            let bottom = if gain {
                0.
            } else {
                n.points
                    .iter()
                    .flat_map(|p| [db(p.s[0].norm()), db(p.s[3].norm())])
                    .filter(|v| v.is_finite())
                    .fold(-5., f64::min)
                    .floor()
            };
            let right = n
                .points
                .iter()
                .flat_map(|p| {
                    let s = p.stability();
                    [s.k, s.mu]
                })
                .filter(|v| v.is_finite())
                .fold(1., f64::max)
                .ceil();
            let xy = |f: f64, v: f64, lo: f64, hi: f64| {
                [
                    60. + ((tx(f) - a) / (b - a).max(1e-20)) as f32 * 460.,
                    490. - ((v - lo) / (hi - lo).max(1e-20)) as f32 * 260.,
                ]
            };
            for i in 0..=5 {
                let y = 490. - 52. * i as f32;
                page.marks
                    .push(Mark::Path(vec![[60., y], [520., y]], [150; 3], 0.5));
                page.marks.push(Mark::Text(
                    [25., y - 4.],
                    format!("{:.2}", bottom + (top - bottom) * i as f64 / 5.),
                    8.,
                    [0; 3],
                ));
                if gain {
                    page.marks.push(Mark::Text(
                        [530., y - 4.],
                        format!("{:.2}", right * i as f64 / 5.),
                        8.,
                        [0; 3],
                    ));
                }
            }
            for (i, name, color) in if gain {
                vec![
                    (0, "MAG", [255, 0, 0]),
                    (1, "MSG", [0, 0, 255]),
                    (2, "K", [0, 150, 0]),
                    (3, "μ", [255, 128, 0]),
                ]
            } else {
                vec![(4, "S11", [0, 0, 255]), (5, "S22", [255, 0, 0])]
            } {
                if i == 5 && n.ports == 1 {
                    continue;
                }
                let points = n
                    .points
                    .iter()
                    .filter_map(|p| {
                        let s = p.stability();
                        if (i == 0 && !s.stable) || (i == 1 && s.stable) {
                            return None;
                        }
                        let value = match i {
                            0 | 1 => s.gain_db,
                            2 => s.k,
                            3 => s.mu,
                            4 => db(p.s[0].norm()),
                            _ => db(p.s[3].norm()),
                        };
                        value.is_finite().then(|| {
                            xy(
                                p.hz,
                                value,
                                if i == 2 || i == 3 { 0. } else { bottom },
                                if i == 2 || i == 3 { right } else { top },
                            )
                        })
                    })
                    .collect();
                page.marks.push(Mark::Path(points, color, 1.));
                page.marks.push(Mark::Text(
                    [65. + i as f32 % 4. * 110., 520.],
                    name.into(),
                    10.,
                    color,
                ));
            }
            page.marks.push(Mark::Text(
                [60., 550.],
                format!(
                    "f [MHz]: {:.2} – {:.2}     [dB] left; K / μ right",
                    n.points[0].hz / 1e6,
                    n.points.last().unwrap().hz / 1e6
                ),
                9.,
                [0; 3],
            ));
            out.push(page);
        }
    } else {
        let mut chart_project = p.clone();
        chart_project.settings.z_grid = selected[1];
        chart_project.settings.y_grid = selected[2];
        chart_project.settings.labels = selected[9];
        if !selected[3] {
            chart_project.elements.clear();
        }
        chart_project.sweep.visible &= selected[10];
        chart_project.circles.retain(|c| {
            selected[if c.q.is_some() {
                4
            } else if c.label.starts_with("VSWR") {
                5
            } else if c.label.starts_with("Gain") {
                7
            } else if c.label.starts_with("Noise") {
                8
            } else {
                6
            }]
        });
        if selected[0] {
            let mut page = Page::new(&p.title);
            page.scene(chart::scene(&chart_project));
            out.push(page);
        }
        if selected[11] {
            let mut page = Page::new("Schematic");
            for (i, e) in p.elements.iter().enumerate() {
                let x = 60. + (i % 5) as f32 * 100.;
                let y = 110. + (i / 5) as f32 * 115.;
                let first_mark = page.marks.len();
                let mut path = vec![[x, y], [x + 30., y]];
                match e.kind {
                    Kind::SeriesR | Kind::ShuntR => path.extend([
                        [x + 30., y - 8.],
                        [x + 65., y - 8.],
                        [x + 65., y + 8.],
                        [x + 30., y + 8.],
                        [x + 30., y],
                        [x + 65., y],
                    ]),
                    Kind::SeriesC | Kind::ShuntC => {
                        path.extend([[x + 43., y], [x + 43., y - 12.], [x + 43., y + 12.]]);
                        page.marks.push(Mark::Path(
                            vec![
                                [x + 51., y - 12.],
                                [x + 51., y + 12.],
                                [x + 51., y],
                                [x + 90., y],
                            ],
                            [0; 3],
                            1.,
                        ));
                    }
                    Kind::SeriesL | Kind::ShuntL => {
                        for k in 0..=60 {
                            path.push([
                                x + 30. + k as f32 * 0.6,
                                y - ((k as f32 / 15. * std::f32::consts::PI).sin().abs()) * 12.,
                            ]);
                        }
                    }
                    _ => path.extend([
                        [x + 30., y - 10.],
                        [x + 65., y - 10.],
                        [x + 65., y + 10.],
                        [x + 30., y + 10.],
                        [x + 30., y],
                        [x + 65., y],
                    ]),
                }
                if !matches!(e.kind, Kind::SeriesC | Kind::ShuntC) {
                    path.push([x + 90., y]);
                }
                page.marks.push(Mark::Path(path, [0; 3], 1.));
                if e.kind.shunt() {
                    for mark in &mut page.marks[first_mark..] {
                        if let Mark::Path(points, _, _) = mark {
                            for p in points {
                                let original = *p;
                                *p = [
                                    x + 47. - (original[1] - y) * 0.6,
                                    y + (original[0] - x) * 0.6,
                                ];
                            }
                        }
                    }
                    page.marks
                        .push(Mark::Path(vec![[x, y], [x + 100., y]], [0; 3], 1.));
                    page.marks.push(Mark::Path(
                        vec![[x + 47., y + 54.], [x + 47., y + 70.]],
                        [0; 3],
                        1.,
                    ));
                } else {
                    page.marks
                        .push(Mark::Path(vec![[x + 90., y], [x + 100., y]], [0; 3], 1.));
                }
                page.marks.push(Mark::Path(
                    vec![[x, y + 70.], [x + 100., y + 70.]],
                    [0; 3],
                    1.,
                ));
                page.marks.push(Mark::Text(
                    [x, y + 80.],
                    format!("{} {}", i + 1, e.kind.label()),
                    7.,
                    [0; 3],
                ));
                page.marks.push(Mark::Text(
                    [x, y + 92.],
                    format!("{:.2} {}", e.value, e.kind.unit()),
                    8.,
                    [0; 3],
                ));
            }
            out.push(page);
        }
        if selected[12] {
            table(
                &mut out,
                "Datapoints",
                formats::point_csv(p, ';').lines().map(str::to_string),
            );
        }
        if selected[13] {
            table(
                &mut out,
                "Circles",
                formats::circle_csv(p).lines().map(str::to_string),
            );
        }
    }
    out
}
pub fn preview(ui: &egui::Ui, page: &Page, paper: Rect) {
    let p = ui.painter().with_clip_rect(paper);
    p.rect_filled(paper, 0., Color32::WHITE);
    let scale = paper.width() / 595.;
    let xy = |v: [f32; 2]| paper.min + vec2(v[0] * scale, v[1] * scale);
    for mark in &page.marks {
        match mark {
            Mark::Path(points, c, w) => {
                p.add(egui::Shape::line(
                    points.iter().copied().map(xy).collect(),
                    Stroke::new((w * scale).max(0.3), Color32::from_rgb(c[0], c[1], c[2])),
                ));
            }
            Mark::Text(pos, text, size, c) => {
                p.text(
                    xy(*pos),
                    egui::Align2::LEFT_TOP,
                    text,
                    FontId::proportional(size * scale),
                    Color32::from_rgb(c[0], c[1], c[2]),
                );
            }
            Mark::Dot(pos, c) => {
                p.circle_filled(xy(*pos), scale * 2., Color32::from_rgb(c[0], c[1], c[2]));
            }
        }
    }
    ui.painter().rect_stroke(
        paper,
        0.,
        Stroke::new(1f32, Color32::BLACK),
        egui::StrokeKind::Outside,
    );
}

#[cfg(not(windows))]
pub fn print(_pages: &[Page]) -> Result<(), String> {
    Err("Native printing requires Windows".into())
}
#[cfg(windows)]
pub fn print(pages: &[Page]) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::GlobalFree, Graphics::Gdi::*, Storage::Xps::*, UI::Controls::Dialogs::*,
    };
    if pages.is_empty() {
        return Err("Select at least one item to print".into());
    }
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // The Windows common dialog owns the printer selection; no print is submitted
    // before the user presses Print there. Restore and release every GDI object.
    unsafe {
        let mut d: PRINTDLGW = std::mem::zeroed();
        d.lStructSize = std::mem::size_of::<PRINTDLGW>() as u32;
        d.Flags = PD_RETURNDC | PD_NOSELECTION | PD_USEDEVMODECOPIESANDCOLLATE;
        d.nFromPage = 1;
        d.nToPage = pages.len() as u16;
        d.nMinPage = 1;
        d.nMaxPage = pages.len() as u16;
        d.nCopies = 1;
        if PrintDlgW(&mut d) == 0 {
            let code = CommDlgExtendedError();
            if !d.hDevMode.is_null() {
                GlobalFree(d.hDevMode);
            }
            if !d.hDevNames.is_null() {
                GlobalFree(d.hDevNames);
            }
            return if code == 0 {
                Ok(())
            } else {
                Err(format!("Windows print dialog: {code:#x}"))
            };
        }
        let dc = d.hDC;
        let title = wide("SmithCraft");
        let info = DOCINFOW {
            cbSize: std::mem::size_of::<DOCINFOW>() as i32,
            lpszDocName: title.as_ptr(),
            ..std::mem::zeroed()
        };
        let scale = (GetDeviceCaps(dc, HORZRES as i32) as f32 / 595.)
            .min(GetDeviceCaps(dc, VERTRES as i32) as f32 / 842.);
        let color = |c: [u8; 3]| c[0] as u32 | ((c[1] as u32) << 8) | ((c[2] as u32) << 16);
        let mut good = StartDocW(dc, &info) > 0;
        let first = if d.Flags & PD_PAGENUMS != 0 {
            d.nFromPage.saturating_sub(1) as usize
        } else {
            0
        };
        let last = if d.Flags & PD_PAGENUMS != 0 {
            d.nToPage as usize
        } else {
            pages.len()
        };
        for page in pages.iter().take(last).skip(first) {
            if !good {
                break;
            }
            good = StartPage(dc) > 0;
            SetBkMode(dc, TRANSPARENT as i32);
            for mark in &page.marks {
                match mark {
                    Mark::Path(points, c, width) => {
                        let pen =
                            CreatePen(PS_SOLID, (*width * scale).round().max(1.) as i32, color(*c));
                        let old = SelectObject(dc, pen);
                        let points: Vec<windows_sys::Win32::Foundation::POINT> = points
                            .iter()
                            .map(|p| windows_sys::Win32::Foundation::POINT {
                                x: (p[0] * scale) as i32,
                                y: (p[1] * scale) as i32,
                            })
                            .collect();
                        if points.len() > 1 {
                            Polyline(dc, points.as_ptr(), points.len() as i32);
                        }
                        SelectObject(dc, old);
                        DeleteObject(pen);
                    }
                    Mark::Text(pos, text, size, c) => {
                        let face = wide("Tahoma");
                        let font = CreateFontW(
                            -(*size * scale).round().max(1.) as i32,
                            0,
                            0,
                            0,
                            400,
                            0,
                            0,
                            0,
                            DEFAULT_CHARSET as u32,
                            0,
                            0,
                            0,
                            0,
                            face.as_ptr(),
                        );
                        let old = SelectObject(dc, font);
                        SetTextColor(dc, color(*c));
                        let text = wide(text);
                        TextOutW(
                            dc,
                            (pos[0] * scale) as i32,
                            (pos[1] * scale) as i32,
                            text.as_ptr(),
                            text.len() as i32 - 1,
                        );
                        SelectObject(dc, old);
                        DeleteObject(font);
                    }
                    Mark::Dot(pos, c) => {
                        let brush = CreateSolidBrush(color(*c));
                        let old = SelectObject(dc, brush);
                        let x = (pos[0] * scale) as i32;
                        let y = (pos[1] * scale) as i32;
                        let r = (2. * scale).max(1.) as i32;
                        Ellipse(dc, x - r, y - r, x + r, y + r);
                        SelectObject(dc, old);
                        DeleteObject(brush);
                    }
                }
            }
            if good {
                good = EndPage(dc) > 0;
            }
        }
        if good {
            good = EndDoc(dc) > 0;
        } else {
            AbortDoc(dc);
        }
        DeleteDC(dc);
        if !d.hDevMode.is_null() {
            GlobalFree(d.hDevMode);
        }
        if !d.hDevNames.is_null() {
            GlobalFree(d.hDevNames);
        }
        if good {
            Ok(())
        } else {
            Err("Windows could not complete the print job".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn print_selection_controls_pages_and_sweep() {
        let p = Project::demo();
        assert!(pages(&p, false, [false; 22]).is_empty());
        let mut selected = [false; 22];
        selected[12] = true;
        let result = pages(&p, false, selected);
        assert!(result.iter().all(|page| page.title == "Datapoints"));
        selected[12] = false;
        selected[0] = true;
        let without = pages(&p, false, selected)[0].marks.len();
        selected[10] = true;
        assert!(pages(&p, false, selected)[0].marks.len() > without);
    }
    #[test]
    fn gain_preview_scales_stability_values_above_two() {
        let p = Project {
            network: Some(
                formats::parse_touchstone(include_str!("../examples/amplifier.s2p"), false)
                    .unwrap(),
            ),
            ..Project::default()
        };
        let mut selected = [false; 22];
        selected[20] = true;
        let result = pages(&p, true, selected);
        assert_eq!(result.len(), 1);
        let green = result[0]
            .marks
            .iter()
            .find_map(|m| match m {
                Mark::Path(p, [0, 150, 0], _) => Some(p),
                _ => None,
            })
            .unwrap();
        assert!(green.len() > 1);
        assert!(
            green
                .iter()
                .all(|p| p[1].is_finite() && p[1] >= 230. && p[1] <= 490.)
        );
    }
}
