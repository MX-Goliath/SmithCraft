use crate::rf::{Element, Kind};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
/// Left-to-right order of the Elements bar in Smith V4.1. The same kind selects
/// both the artwork and the insertion action, including the two RLC topologies.
pub const TOOLBAR_KINDS: [Kind; 12] = [
    Kind::SeriesC,
    Kind::SeriesL,
    Kind::SeriesR,
    Kind::Line,
    Kind::Transformer,
    Kind::SeriesParallelRlc,
    Kind::ShuntC,
    Kind::ShuntL,
    Kind::ShuntR,
    Kind::OpenStub,
    Kind::ShortStub,
    Kind::ShuntSeriesRlc,
];

/// Pixel artwork restored from the user-supplied toolbar reference. Each cell
/// is 40×40 reference pixels (including the labels and both circuit rails).
/// Keep this separate from schematic symbols: their orientation and layout differ.
pub fn toolbar_symbol(p: &egui::Painter, r: Rect, e: &Element) {
    let id = egui::Id::new("element_toolbar_artwork");
    let texture = p
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(id));
    let texture = texture.unwrap_or_else(|| {
        let pixels = include_bytes!("../assets/element-toolbar.pixels")
            .iter()
            .map(|&index| match index {
                1 => Color32::BLACK,
                2 => Color32::from_gray(192),
                3 => Color32::WHITE,
                4 => Color32::from_gray(16),
                _ => Color32::TRANSPARENT,
            })
            .collect();
        let texture = p.ctx().load_texture(
            "Smith Elements",
            egui::ColorImage::new([480, 40], pixels),
            egui::TextureOptions {
                minification: egui::TextureFilter::Linear,
                ..egui::TextureOptions::NEAREST
            },
        );
        p.ctx()
            .data_mut(|data| data.insert_temp(id, texture.clone()));
        texture
    });
    let index = TOOLBAR_KINDS
        .iter()
        .position(|&kind| kind == e.kind)
        .unwrap();
    let uv = Rect::from_min_max(
        egui::pos2(index as f32 / 12., 0.),
        egui::pos2((index + 1) as f32 / 12., 1.),
    );
    // The reference is at 150%: keep its 40 physical pixels at this DPI.
    let size = egui::vec2(40. / 1.5, 40. / 1.5);
    let min = r.center() - size * 0.5;
    let scale = p.ctx().pixels_per_point();
    let min = egui::pos2(
        (min.x * scale).round() / scale,
        (min.y * scale).round() / scale,
    );
    let target = Rect::from_min_size(min, size);
    p.image(texture.id(), target, uv, Color32::WHITE);
}
pub fn oriented_symbol(p: &egui::Painter, r: Rect, e: &Element, vertical: bool) {
    let center = r.center();
    let stroke = Stroke::new(1.0_f32, Color32::BLACK);
    let map = |x: f32, y: f32| {
        let d = if e.kind.shunt() {
            egui::vec2(-y, x)
        } else {
            egui::vec2(x, y)
        };
        center + if vertical { egui::vec2(d.y, d.x) } else { d }
    };
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([map(a.0, a.1), map(b.0, b.1)], stroke);
    };
    match e.kind {
        Kind::SeriesC | Kind::ShuntC => {
            line((-11., 0.), (-2., 0.));
            line((2., 0.), (11., 0.));
            line((-2., -7.), (-2., 7.));
            line((2., -7.), (2., 7.));
        }
        Kind::SeriesL | Kind::ShuntL => {
            let points = (0..=40)
                .map(|i| {
                    map(
                        -9. + 18. * i as f32 / 40.,
                        -3. * (i as f32 / 40. * std::f32::consts::PI * 3.).sin().abs(),
                    )
                })
                .collect();
            p.add(egui::Shape::line(points, stroke));
            line((-11., 0.), (-9., 0.));
            line((9., 0.), (11., 0.));
        }
        Kind::SeriesR | Kind::ShuntR => {
            let a = map(-7., -3.);
            let b = map(7., 3.);
            p.rect_stroke(
                Rect::from_two_pos(a, b),
                0.,
                stroke,
                egui::StrokeKind::Inside,
            );
            line((-10., 0.), (-7., 0.));
            line((7., 0.), (10., 0.));
        }
        Kind::Line => {
            line((-10., -6.), (10., -6.));
            line((-10., 6.), (10., 6.));
            line((-6., -6.), (-6., 6.));
            line((6., -6.), (6., 6.));
        }
        Kind::OpenStub | Kind::ShortStub => {
            line((-8., -3.), (8., -3.));
            line((-8., 3.), (8., 3.));
            if e.kind == Kind::ShortStub {
                line((8., -3.), (8., 3.));
            }
        }
        Kind::Transformer => {
            for x in [-3., 3.] {
                let points = (0..=30)
                    .map(|i| {
                        map(
                            x + 2. * (i as f32 / 30. * std::f32::consts::PI * 3.).sin().abs(),
                            -8. + 16. * i as f32 / 30.,
                        )
                    })
                    .collect();
                p.add(egui::Shape::line(points, stroke));
            }
        }
        Kind::SeriesParallelRlc => {
            line((-11., 0.), (-9., 0.));
            line((9., 0.), (11., 0.));
            line((-9., -7.), (-9., 7.));
            line((9., -7.), (9., 7.));
            line((-9., -7.), (-5., -7.));
            line((5., -7.), (9., -7.));
            for (a, b) in [
                ((-5., -9.), (5., -9.)),
                ((5., -9.), (5., -5.)),
                ((5., -5.), (-5., -5.)),
                ((-5., -5.), (-5., -9.)),
            ] {
                line(a, b);
            }
            let pts = (0..=40)
                .map(|i| {
                    map(
                        -9. + 18. * i as f32 / 40.,
                        -2. * (i as f32 / 40. * std::f32::consts::PI * 3.).sin().abs(),
                    )
                })
                .collect();
            p.add(egui::Shape::line(pts, stroke));
            line((-9., 7.), (-2., 7.));
            line((2., 7.), (9., 7.));
            line((-2., 4.), (-2., 10.));
            line((2., 4.), (2., 10.));
        }
        Kind::ShuntSeriesRlc => {
            line((-11., 0.), (-9., 0.));
            for (a, b) in [
                ((-9., -3.), (-3., -3.)),
                ((-3., -3.), (-3., 3.)),
                ((-3., 3.), (-9., 3.)),
                ((-9., 3.), (-9., -3.)),
            ] {
                line(a, b);
            }
            let pts = (0..=32)
                .map(|i| {
                    map(
                        -3. + 9. * i as f32 / 32.,
                        -2. * (i as f32 / 32. * std::f32::consts::PI * 3.).sin().abs(),
                    )
                })
                .collect();
            p.add(egui::Shape::line(pts, stroke));
            line((6., -3.), (6., 3.));
            line((8., -3.), (8., 3.));
            line((8., 0.), (11., 0.));
        }
    }
}
/// Draw circuit symbols as geometry so they do not depend on font glyph support.
pub fn draw(p: &egui::Painter, rect: Rect, e: &Element, index: usize) {
    let color = if e.enabled {
        Color32::from_rgb(40, 65, 95)
    } else {
        Color32::GRAY
    };
    let stroke = Stroke::new(1.4_f32, color);
    let x = rect.center().x;
    let y = rect.top() + 18.;
    let shunt = e.kind.shunt();
    let map = |u: f32, v: f32| {
        if shunt {
            Pos2::new(x - v, y + 20. + u * 0.65)
        } else {
            Pos2::new(x + u, y + v)
        }
    };
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([map(a.0, a.1), map(b.0, b.1)], stroke);
    };
    if shunt {
        p.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            stroke,
        );
        p.line_segment([Pos2::new(x, y), map(-20., 0.)], stroke);
    } else {
        p.line_segment([Pos2::new(rect.left(), y), map(-20., 0.)], stroke);
        p.line_segment([map(20., 0.), Pos2::new(rect.right(), y)], stroke);
    }
    match e.kind {
        Kind::SeriesC | Kind::ShuntC => {
            line((-20., 0.), (-3., 0.));
            line((3., 0.), (20., 0.));
            line((-3., -10.), (-3., 10.));
            line((3., -10.), (3., 10.));
        }
        Kind::SeriesL | Kind::ShuntL => {
            line((-20., 0.), (-15., 0.));
            line((15., 0.), (20., 0.));
            let pts = (0..=120)
                .map(|i| {
                    let u = -15. + 30. * i as f32 / 120.;
                    let v = -7. * (i as f32 / 120. * std::f32::consts::PI * 4.).sin().abs();
                    map(u, v)
                })
                .collect();
            p.add(egui::Shape::line(pts, stroke));
        }
        Kind::SeriesR | Kind::ShuntR => {
            let pts = (-4..=4)
                .map(|i| {
                    map(
                        i as f32 * 5.,
                        if i == -4 || i == 4 {
                            0.
                        } else if i % 2 == 0 {
                            -6.
                        } else {
                            6.
                        },
                    )
                })
                .collect();
            p.add(egui::Shape::line(pts, stroke));
        }
        Kind::Transformer => {
            for offset in [-8., 8.] {
                let pts = (0..=100)
                    .map(|i| {
                        let v = -10. + 20. * i as f32 / 100.;
                        map(
                            offset + 3. * (i as f32 / 100. * std::f32::consts::PI * 3.).sin().abs(),
                            v,
                        )
                    })
                    .collect();
                p.add(egui::Shape::line(pts, stroke));
            }
            line((-20., 0.), (-11., 0.));
            line((11., 0.), (20., 0.));
        }
        Kind::OpenStub | Kind::ShortStub => {
            line((-20., -4.), (17., -4.));
            line((-20., 4.), (17., 4.));
            if e.kind == Kind::ShortStub {
                line((17., -4.), (17., 4.));
            }
        }
        _ => {
            let points = vec![
                map(-18., -9.),
                map(18., -9.),
                map(18., 9.),
                map(-18., 9.),
                map(-18., -9.),
            ];
            p.add(egui::Shape::line(points, stroke));
            p.text(
                map(0., 0.),
                egui::Align2::CENTER_CENTER,
                if e.kind == Kind::Line { "TL" } else { "RLC" },
                egui::FontId::proportional(10.),
                color,
            );
        }
    }
    if shunt && !matches!(e.kind, Kind::OpenStub) {
        let end = map(23., 0.);
        p.line_segment([map(20., 0.), end], stroke);
        for i in 0..3 {
            let yy = end.y + i as f32 * 3.;
            p.line_segment(
                [
                    Pos2::new(x - 8. + i as f32 * 2., yy),
                    Pos2::new(x + 8. - i as f32 * 2., yy),
                ],
                stroke,
            );
        }
    }
    p.text(
        Pos2::new(x, rect.bottom() - 24.),
        egui::Align2::CENTER_CENTER,
        format!("{} {}", index + 1, e.kind.label()),
        egui::FontId::proportional(10.),
        color,
    );
    p.text(
        Pos2::new(x, rect.bottom() - 10.),
        egui::Align2::CENTER_CENTER,
        format!(
            "{:.2} {}",
            e.value,
            if matches!(e.kind, Kind::Line | Kind::OpenStub | Kind::ShortStub) {
                "λ"
            } else {
                e.kind.unit()
            }
        ),
        egui::FontId::proportional(11.),
        color,
    );
}
