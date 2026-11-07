use crate::{project::*, rf::*};
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use std::f64::consts::TAU;
#[derive(Clone)]
pub enum Primitive {
    Path(Vec<[f64; 2]>, [u8; 3], f32),
    Dot([f64; 2], [u8; 3], bool),
    Text([f64; 2], String, [u8; 3]),
}
pub fn grid(p: &Project) -> Vec<Primitive> {
    let mut scene = Vec::new();
    let set = &p.settings;
    for (enabled, sign, color) in [
        (set.z_grid, 1., set.z_color),
        (set.y_grid, -1., set.y_color),
    ] {
        if !enabled {
            continue;
        }
        for r in [0., 0.2, 0.5, 1., 2., 5., 10.] {
            let pts = (0..=600)
                .map(|i| {
                    let t = -std::f64::consts::FRAC_PI_2
                        + std::f64::consts::PI * (0.00001 + 0.99998 * i as f64 / 600.);
                    let g = gamma(c(r, t.tan()), 1.) * sign;
                    [g.re, g.im]
                })
                .collect();
            scene.push(Primitive::Path(pts, color, set.grid_width));
        }
        for x in [-10., -5., -2., -1., -0.5, -0.2, 0.2, 0.5, 1., 2., 5., 10.] {
            let pts = (0..=300)
                .map(|i| {
                    let g = if set.expanded {
                        c(1., 1. / x) + C::from_polar(1. / x.abs(), TAU * i as f64 / 300.)
                    } else {
                        let r = 50. * (i as f64 / 300.).powi(3);
                        gamma(c(r, x), 1.)
                    } * sign;
                    [g.re, g.im]
                })
                .collect();
            scene.push(Primitive::Path(pts, color, set.grid_width));
            if set.labels && x > 0. {
                let g = gamma(c(0., x), 1.) * sign;
                scene.push(Primitive::Text(
                    [g.re * 1.045, g.im * 1.045],
                    format!(
                        "{:.2}{}",
                        if sign > 0. {
                            x * p.z0
                        } else {
                            1000. * x / p.z0
                        },
                        if sign > 0. { "" } else { "m" }
                    ),
                    color,
                ));
            }
        }
        if set.labels {
            for r in [0.2, 0.5, 1., 2., 5.] {
                let g = gamma(c(r, 0.), 1.) * sign;
                scene.push(Primitive::Text(
                    [g.re, 0.025 * sign],
                    format!(
                        "{:.2}{}",
                        if sign > 0. {
                            r * p.z0
                        } else {
                            1000. * r / p.z0
                        },
                        if sign > 0. { "" } else { "m" }
                    ),
                    color,
                ));
            }
        }
    }
    scene.push(Primitive::Path(
        (0..=600)
            .map(|i| {
                let t = TAU * i as f64 / 600.;
                [t.cos(), t.sin()]
            })
            .collect(),
        [30, 35, 42],
        1.2,
    ));
    scene.push(Primitive::Path(
        vec![[-1., 0.], [1., 0.]],
        [100, 110, 120],
        0.5,
    ));
    scene
}
pub fn scene(p: &Project) -> Vec<Primitive> {
    scene_selected(p, true)
}
fn scene_selected(p: &Project, circuit: bool) -> Vec<Primitive> {
    let mut s = grid(p);
    for circle in p.circles.iter().filter(|x| x.visible) {
        let color = if circle.highlighted {
            [230, 130, 20]
        } else {
            [137, 65, 165]
        };
        let width = if circle.highlighted { 3. } else { 1.5 };
        if let Some(q) = circle.q {
            for sign in [-1., 1.] {
                let pts = (0..=500)
                    .map(|i| {
                        let r = 100. * (i as f64 / 500.).powi(3);
                        let g = gamma(c(r, sign * r * q), 1.);
                        [g.re, g.im]
                    })
                    .collect();
                s.push(Primitive::Path(pts, color, width));
            }
        } else {
            s.push(Primitive::Path(
                (0..=500)
                    .map(|i| {
                        let t = TAU * i as f64 / 500.;
                        [
                            circle.center.re + circle.radius * t.cos(),
                            circle.center.im + circle.radius * t.sin(),
                        ]
                    })
                    .collect(),
                color,
                width,
            ));
        }
    }
    for (i, d) in p.points.iter().enumerate() {
        let g = gamma(d.z, p.z0);
        s.push(Primitive::Dot([g.re, g.im], [15, 95, 180], i == p.start));
        if p.settings.labels {
            s.push(Primitive::Text(
                [g.re + 0.025, g.im + 0.03],
                d.name.clone(),
                [15, 70, 145],
            ));
        }
    }
    if circuit && let Some(a) = p.active() {
        let mut z = a.z;
        for (i, e) in p.elements.iter().enumerate() {
            let pts = (0..=100)
                .map(|j| {
                    let g = gamma(e.apply_fraction(z, a.hz, j as f64 / 100.), p.z0);
                    [g.re, g.im]
                })
                .collect();
            s.push(Primitive::Path(
                pts,
                p.settings.trace_color,
                p.settings.trace_width,
            ));
            z = e.apply(z, a.hz);
            let g = gamma(z, p.z0);
            s.push(Primitive::Dot([g.re, g.im], p.settings.trace_color, true));
            if p.settings.labels {
                s.push(Primitive::Text(
                    [g.re + 0.025, g.im - 0.035],
                    format!("TP{}", i + 1),
                    p.settings.trace_color,
                ));
            }
        }
    }
    let sweep = p.swept();
    if !sweep.is_empty() {
        s.push(Primitive::Path(
            sweep
                .iter()
                .map(|(_, z)| {
                    let g = gamma(*z, p.z0);
                    [g.re, g.im]
                })
                .collect(),
            [20, 155, 90],
            2.,
        ));
        for (i, (_, z)) in sweep.iter().enumerate() {
            if i % ((sweep.len() / 30).max(1)) == 0 {
                let g = gamma(*z, p.z0);
                s.push(Primitive::Dot([g.re, g.im], [20, 155, 90], false));
            }
        }
    }
    s
}
pub struct ChartResponse {
    pub response: egui::Response,
    pub gamma: Option<C>,
    pub radius: f32,
}
pub fn paint(ui: &mut egui::Ui, scene: &[Primitive], zoom: f32, size: Vec2) -> ChartResponse {
    let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = response.rect;
    painter.rect_filled(rect, 0., Color32::WHITE);
    let radius = (rect.width().min(rect.height()) * 0.445) * zoom;
    let center = rect.center();
    let point = |p: [f64; 2]| {
        Pos2::new(
            center.x + p[0] as f32 * radius,
            center.y - p[1] as f32 * radius,
        )
    };
    for item in scene {
        match item {
            Primitive::Path(pts, color, width) => {
                let mut segment = Vec::new();
                for p in pts {
                    if p[0].is_finite() && p[1].is_finite() && p[0].abs() < 1e5 && p[1].abs() < 1e5
                    {
                        segment.push(point(*p));
                    } else if segment.len() > 1 {
                        painter.add(egui::Shape::line(
                            std::mem::take(&mut segment),
                            Stroke::new(*width, Color32::from_rgb(color[0], color[1], color[2])),
                        ));
                    }
                }
                if segment.len() > 1 {
                    painter.add(egui::Shape::line(
                        segment,
                        Stroke::new(*width, Color32::from_rgb(color[0], color[1], color[2])),
                    ));
                }
            }
            Primitive::Dot(pos, color, square) => {
                let pt = point(*pos);
                let color = Color32::from_rgb(color[0], color[1], color[2]);
                if *square {
                    painter.rect_filled(
                        egui::Rect::from_center_size(pt, Vec2::splat(7.)),
                        0.,
                        color,
                    );
                } else {
                    painter.circle_filled(pt, 2.5, color);
                }
            }
            Primitive::Text(pos, text, color) => {
                painter.text(
                    point(*pos),
                    egui::Align2::LEFT_BOTTOM,
                    text,
                    egui::FontId::proportional(12.),
                    Color32::from_rgb(color[0], color[1], color[2]),
                );
            }
        }
    }
    let g = response.hover_pos().map(|pt| {
        c(
            ((pt.x - center.x) / radius) as f64,
            ((center.y - pt.y) / radius) as f64,
        )
    });
    ChartResponse {
        response,
        gamma: g,
        radius,
    }
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn svg(p: &Project) -> String {
    scene_svg(scene(p))
}
pub fn scene_svg(scene: Vec<Primitive>) -> String {
    let mut out="<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"1000\" viewBox=\"0 0 1000 1000\"><rect width=\"1000\" height=\"1000\" fill=\"white\"/>".to_string();
    for item in scene {
        match item{Primitive::Path(pts,rgb,width)=>{let points=pts.into_iter().filter(|p|p[0].is_finite()&&p[1].is_finite()).map(|p|format!("{:.3},{:.3}",500.+p[0]*435.,500.-p[1]*435.)).collect::<Vec<_>>().join(" ");out.push_str(&format!("<polyline points=\"{points}\" fill=\"none\" stroke=\"rgb({},{},{})\" stroke-width=\"{width}\"/>",rgb[0],rgb[1],rgb[2]));},Primitive::Dot(pos,rgb,_)=>out.push_str(&format!("<circle cx=\"{}\" cy=\"{}\" r=\"3\" fill=\"rgb({},{},{})\"/>",500.+pos[0]*435.,500.-pos[1]*435.,rgb[0],rgb[1],rgb[2])),Primitive::Text(pos,text,rgb)=>out.push_str(&format!("<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"13\" fill=\"rgb({},{},{})\">{}</text>",500.+pos[0]*435.,500.-pos[1]*435.,rgb[0],rgb[1],rgb[2],escape(&text)))}
    }
    out.push_str("</svg>");
    out
}
pub fn network_svg(n: &crate::formats::Network, settings: &Settings) -> String {
    network_svg_selected(n, settings, [true; 4])
}
pub fn network_svg_selected(
    n: &crate::formats::Network,
    settings: &Settings,
    selected: [bool; 4],
) -> String {
    let p = Project {
        z0: n.z0,
        settings: settings.clone(),
        ..Project::default()
    };
    let mut out = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"1200\" viewBox=\"0 0 1200 1200\"><rect width=\"1200\" height=\"1200\" fill=\"white\"/>",
    );
    for (slot, j) in [0, 3, 1, 2].iter().enumerate() {
        if !selected[[0, 2, 1, 3][*j]] {
            continue;
        }
        if n.ports == 1 && *j != 0 {
            continue;
        }
        let mut scene = if *j == 0 || *j == 3 {
            grid(&p)
        } else {
            let mut result = vec![];
            for radius in [0.25, 0.5, 0.75, 1.] {
                result.push(Primitive::Path(
                    (0..=300)
                        .map(|i| {
                            let t = TAU * i as f64 / 300.;
                            [radius * t.cos(), radius * t.sin()]
                        })
                        .collect(),
                    [160, 170, 180],
                    0.7,
                ));
            }
            result
        };
        let scale = if *j == 1 || *j == 2 {
            n.points
                .iter()
                .map(|p| p.s[*j].norm())
                .fold(0f64, f64::max)
                .max(1e-20)
                * 1.05
        } else {
            1.
        };
        scene.push(Primitive::Path(
            n.points
                .iter()
                .map(|p| [p.s[*j].re / scale, p.s[*j].im / scale])
                .collect(),
            [25, 100, 190],
            2.5,
        ));
        let svg = scene_svg(scene).replacen(
            "width=\"1000\" height=\"1000\"",
            &format!(
                "x=\"{}\" y=\"{}\" width=\"600\" height=\"570\"",
                slot % 2 * 600,
                slot / 2 * 600 + 30
            ),
            1,
        );
        out.push_str(&format!("<text x=\"{}\" y=\"{}\" font-family=\"Arial\" font-size=\"20\">S{}{} (radius {:.2})</text>",slot%2*600+20,slot/2*600+25,if *j==0||*j==2{1}else{2},if *j==0||*j==1{1}else{2},scale));
        out.push_str(&svg);
    }
    out.push_str("</svg>");
    out
}
pub fn network_report(n: &crate::formats::Network, settings: &Settings) -> String {
    network_report_selected(n, settings, [true; 8])
}
pub fn network_report_selected(
    n: &crate::formats::Network,
    settings: &Settings,
    selected: [bool; 8],
) -> String {
    let mut out = format!(
        "<!doctype html><html><meta charset=\"utf-8\"><title>S-Plot {}</title><style>body{{font:13px Arial;max-width:1000px;margin:24px auto}}svg{{width:100%;height:auto}}table{{border-collapse:collapse;font-size:11px;width:100%}}th,td{{border:1px solid #ccc;padding:4px}}@media print{{button{{display:none}}}}h2{{break-before:auto}}</style><button onclick=\"print()\">Print / Save PDF</button><h1>S-Plot {}</h1><p>Z0 {} ohm; {} records</p>{}",
        escape(&n.name),
        escape(&n.name),
        n.z0,
        n.points.len(),
        network_svg_selected(
            n,
            settings,
            [selected[0], selected[1], selected[2], selected[3]]
        )
    );
    if selected[4] {
        out.push_str("<h2>File info</h2><pre>");
        out.push_str(&escape(&n.source));
        out.push_str("</pre>");
    }
    for (gain, title) in [(true, "Gain and stability"), (false, "Return loss [dB]")] {
        if (gain && !selected[6]) || (!gain && !selected[7]) {
            continue;
        }
        if gain && n.ports == 1 {
            continue;
        }
        out.push_str(&format!("<h2>{title}</h2><svg viewBox=\"0 0 1000 300\"><defs><clipPath id=\"plot-{gain}\"><rect x=\"55\" y=\"15\" width=\"850\" height=\"250\"/></clipPath></defs><rect x=\"55\" y=\"15\" width=\"850\" height=\"250\" fill=\"white\" stroke=\"#aaa\"/>"));
        let x = |f: f64| if settings.log_frequency { f.log10() } else { f };
        let xmin = x(n.points[0].hz);
        let xmax = x(n.points.last().unwrap().hz);
        let curves: Vec<(String, Vec<[f64; 2]>)> = if gain {
            ["MAG [dB]", "MSG [dB]", "K", "μ"]
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    (
                        name.to_string(),
                        n.points
                            .iter()
                            .filter_map(|p| {
                                let st = p.stability();
                                if (i == 0 && !st.stable) || (i == 1 && st.stable) {
                                    return None;
                                }
                                let y = match i {
                                    0 | 1 => st.gain_db,
                                    2 => st.k,
                                    _ => st.mu,
                                };
                                y.is_finite().then_some([x(p.hz), y])
                            })
                            .collect(),
                    )
                })
                .collect()
        } else {
            [0, 3]
                .iter()
                .filter(|j| **j == 0 || n.ports == 2)
                .map(|j| {
                    (
                        if *j == 0 { "RL11" } else { "RL22" }.into(),
                        n.points
                            .iter()
                            .map(|p| [x(p.hz), db(p.s[*j].norm())])
                            .collect(),
                    )
                })
                .collect()
        };
        let ymin = if gain {
            0.
        } else {
            curves
                .iter()
                .flat_map(|(_, pts)| pts.iter().map(|p| p[1]))
                .filter(|v| v.is_finite())
                .fold(-5., f64::min)
        };
        let ymax = if gain {
            curves
                .iter()
                .take(2)
                .flat_map(|(_, pts)| pts.iter().map(|p| p[1]))
                .filter(|v| v.is_finite())
                .fold(10., f64::max)
        } else {
            0.
        };
        let stability_max = curves
            .iter()
            .skip(2)
            .flat_map(|(_, p)| p.iter().map(|p| p[1]))
            .filter(|v| v.is_finite())
            .fold(1f64, f64::max)
            .ceil();
        for (i, (name, pts)) in curves.iter().enumerate() {
            let color = if gain {
                ["#ff0000", "#0000ff", "#009600", "#ff8000"][i]
            } else {
                ["#0000ff", "#ff0000"][i]
            };
            let (lo, hi) = if gain && i >= 2 {
                (0., stability_max)
            } else {
                (ymin, ymax)
            };
            let points = pts
                .iter()
                .filter(|p| p[1].is_finite())
                .map(|p| {
                    format!(
                        "{:.3},{:.3}",
                        55. + 850. * (p[0] - xmin) / (xmax - xmin).max(1e-30),
                        265. - 250. * (p[1] - lo) / (hi - lo).max(1e-30)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            out.push_str(&format!("<polyline clip-path=\"url(#plot-{gain})\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2\" points=\"{points}\"/><text x=\"{}\" y=\"288\" font-size=\"14\" fill=\"{color}\">{name}</text>",60+i*180));
        }
        if gain {
            out.push_str(&format!("<text x=\"915\" y=\"25\" font-size=\"12\">{stability_max:.2} K / μ</text><text x=\"915\" y=\"265\" font-size=\"12\">0.00</text>"));
        }
        out.push_str(&format!("<text x=\"2\" y=\"25\" font-size=\"12\">{ymax:.2}</text><text x=\"2\" y=\"265\" font-size=\"12\">{ymin:.2}</text></svg>"));
    }
    if selected[5] {
        out.push_str("<h2>Gain table</h2><table><tr><th>MHz</th><th>S11 mag/phase</th><th>S21 mag/phase</th><th>S12 mag/phase</th><th>S22 mag/phase</th><th>MAG/MSG dB</th><th>K</th><th>Mu</th><th>RL11 dB</th><th>RL22 dB</th></tr>");
        for p in &n.points {
            let st = p.stability();
            out.push_str(&format!("<tr><td>{:.2}</td>", p.hz / 1e6));
            for s in p.s {
                out.push_str(&format!(
                    "<td>{:.2} / {:.2}°</td>",
                    s.norm(),
                    s.arg().to_degrees()
                ));
            }
            for v in [
                st.gain_db,
                st.k,
                st.mu,
                -db(p.s[0].norm()),
                -db(p.s[3].norm()),
            ] {
                out.push_str(&format!("<td>{v:.2}</td>"));
            }
            out.push_str("</tr>");
        }
        out.push_str("</table>");
    }
    out.push_str("</html>");
    out
}
pub fn report(p: &Project) -> String {
    report_selected(p, [true; 4], true)
}
pub fn report_selected(p: &Project, selected: [bool; 4], circuit: bool) -> String {
    let mut out = format!(
        "<!doctype html><html><meta charset=\"utf-8\"><title>{}</title><style>body{{font:13px Arial;max-width:1000px;margin:24px auto}}svg{{width:75%;height:auto}}pre{{white-space:pre-wrap;font-size:10px}}@media print{{button{{display:none}}}}</style><button onclick=\"print()\">Print / Save PDF</button><h1>{}</h1><p>Z0 = {} ohm</p>",
        escape(&p.title),
        escape(&p.title),
        p.z0
    );
    if selected[0] {
        out.push_str(&scene_svg(scene_selected(p, circuit)));
    }
    if selected[1] {
        out.push_str("<h2>Schematic</h2>");
        out.push_str(&schematic_svg(p));
        out.push_str("<ol>");
        for e in &p.elements {
            out.push_str(&format!(
                "<li>{}: {} {}; Zline {} ohm; Er {}; loss {} dB/m</li>",
                e.kind.label(),
                e.value,
                e.kind.unit(),
                e.z0,
                e.er,
                e.loss
            ));
        }
        out.push_str("</ol>");
    }
    if selected[2] {
        out.push_str("<h2>Datapoints and sweep</h2><pre>");
        out.push_str(&escape(&crate::formats::point_csv(p, ';')));
        out.push_str("</pre>");
    }
    if selected[3] {
        out.push_str("<h2>Circles</h2><pre>");
        out.push_str(&escape(&crate::formats::circle_csv(p)));
        out.push_str("</pre>");
    }
    out.push_str("</html>");
    out
}

pub fn schematic_svg(p: &Project) -> String {
    let columns = 6;
    let rows = p.elements.len().div_ceil(columns).max(1);
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 {}\"><rect width=\"1000\" height=\"{}\" fill=\"white\"/>",
        rows * 140,
        rows * 140
    );
    for (i, e) in p.elements.iter().enumerate() {
        let x = 30 + (i % columns) * 160;
        let y = 35 + (i / columns) * 140;
        let stroke = if e.enabled { "#264560" } else { "#aaa" };
        out.push_str(&format!("<g transform=\"translate({x},{y})\" stroke=\"{stroke}\" fill=\"none\" stroke-width=\"2\"><path d=\"M0 0H140\"/>"));
        let shunt = e.kind.shunt();
        let transform = if shunt {
            "translate(70,25) rotate(90) scale(.7)"
        } else {
            "translate(70,0)"
        };
        if shunt {
            out.push_str("<path d=\"M70 0V12 M60 50H80 M63 54H77 M66 58H74\"/>");
        }
        out.push_str(&format!("<g transform=\"{transform}\"><rect x=\"-23\" y=\"-12\" width=\"46\" height=\"24\" fill=\"white\" stroke=\"none\"/>"));
        let path = match e.kind {
            Kind::SeriesC | Kind::ShuntC => "M-23 0H-4 M-4 -12V12 M4 -12V12 M4 0H23",
            Kind::SeriesL | Kind::ShuntL => {
                "M-23 0H-18 C-18 -15 -9 -15 -9 0 C-9 -15 0 -15 0 0 C0 -15 9 -15 9 0 C9 -15 18 -15 18 0H23"
            }
            Kind::SeriesR | Kind::ShuntR => "M-23 0L-18 -7 -12 7 -6 -7 0 7 6 -7 12 7 18 0H23",
            _ => "M-23 0H-18V-10H18V10H-18V0 M18 0H23",
        };
        out.push_str(&format!("<path d=\"{path}\"/></g><text x=\"70\" y=\"82\" text-anchor=\"middle\" font-family=\"Arial\" font-size=\"12\" fill=\"{stroke}\" stroke=\"none\">{} {}</text><text x=\"70\" y=\"100\" text-anchor=\"middle\" font-family=\"Arial\" font-size=\"12\" fill=\"{stroke}\" stroke=\"none\">{:.2} {}</text></g>",i+1,escape(e.kind.label()),e.value,escape(e.kind.unit())));
    }
    out.push_str("</svg>");
    out
}
