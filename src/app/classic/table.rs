//! WinForms-like tables shared by the MDI data windows and import dialogs.
use super::*;
#[derive(Clone)]
pub(super) enum Cell {
    Text(String),
    Check(Option<bool>),
}
impl From<String> for Cell {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}
pub(super) enum Event {
    DoubleClick(usize),
    Toggle(usize, usize),
}
pub(super) fn show(
    ui: &mut Ui,
    key: &str,
    columns: &[(&str, f32)],
    rows: &[(usize, Vec<Cell>)],
    selected: &mut Vec<usize>,
) -> Vec<Event> {
    let id = ui.make_persistent_id(key);
    let mut widths = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Vec<f32>>(id))
        .filter(|w| w.len() == columns.len())
        .unwrap_or_else(|| columns.iter().map(|c| c.1).collect());
    let mut events = vec![];
    egui::ScrollArea::both()
        .id_salt(id)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let width = widths.iter().sum::<f32>().max(ui.available_width());
            let height = (18. + rows.len() as f32 * 15.).max(ui.available_height());
            let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
            response.context_menu(|ui| {
                if ui.button("Select all").clicked() {
                    *selected = rows.iter().map(|r| r.0).collect();
                    ui.close();
                }
                if ui.button("Unselect all").clicked() {
                    selected.clear();
                    ui.close();
                }
            });
            let p = ui.painter();
            p.rect_filled(rect, 0., Color32::WHITE);
            let mut left = rect.left();
            let mut cells = vec![];
            for (i, (name, _)) in columns.iter().enumerate() {
                let w = if i + 1 == columns.len() {
                    (rect.right() - left).max(widths[i])
                } else {
                    widths[i]
                };
                let r = Rect::from_min_size(pos2(left, rect.top()), vec2(w, 18.));
                p.rect_filled(r, 0., Color32::from_gray(246));
                p.rect_stroke(
                    r,
                    0.,
                    Stroke::new(0.5f32, Color32::from_gray(180)),
                    StrokeKind::Inside,
                );
                p.with_clip_rect(r.intersect(ui.clip_rect())).text(
                    r.left_center() + vec2(3., 0.),
                    Align2::LEFT_CENTER,
                    *name,
                    FontId::proportional(11.),
                    Color32::BLACK,
                );
                cells.push((left, w));
                left += w;
            }
            for (line, (row, cells_text)) in rows.iter().enumerate() {
                let rr = Rect::from_min_size(
                    pos2(rect.left(), rect.top() + 18. + line as f32 * 15.),
                    vec2(width, 15.),
                );
                if !rr.intersects(ui.clip_rect()) {
                    continue;
                }
                let active = selected.contains(row);
                if active {
                    p.rect_filled(rr, 0., Color32::from_rgb(0, 120, 215));
                }
                let hit = ui.interact(rr, id.with(("row", row)), Sense::click());
                if hit.clicked() {
                    let modifiers = ui.input(|i| i.modifiers);
                    if modifiers.shift {
                        let anchor = selected.first().copied().unwrap_or(*row);
                        let a = rows.iter().position(|r| r.0 == anchor).unwrap_or(line);
                        *selected = rows[a.min(line)..=a.max(line)]
                            .iter()
                            .map(|r| r.0)
                            .collect();
                    } else if modifiers.ctrl {
                        if selected.contains(row) {
                            selected.retain(|v| v != row);
                        } else {
                            selected.push(*row);
                        }
                    } else {
                        *selected = vec![*row];
                    }
                }
                if hit.double_clicked() {
                    events.push(Event::DoubleClick(*row));
                }
                for (column, cell) in cells_text.iter().enumerate() {
                    let (x, w) = cells[column];
                    let r = Rect::from_min_size(pos2(x, rr.top()), vec2(w, 15.));
                    let c = if active {
                        Color32::WHITE
                    } else {
                        Color32::BLACK
                    };
                    let cp = p.with_clip_rect(r.shrink2(vec2(2., 0.)).intersect(ui.clip_rect()));
                    match cell {
                        Cell::Text(text) => {
                            cp.text(
                                r.left_center() + vec2(3., 0.),
                                Align2::LEFT_CENTER,
                                text,
                                FontId::proportional(11.),
                                c,
                            );
                        }
                        Cell::Check(Some(on)) => {
                            let square = Rect::from_center_size(r.center(), vec2(7., 7.));
                            cp.rect_filled(square, 0., Color32::WHITE);
                            cp.rect_stroke(
                                square,
                                0.,
                                Stroke::new(0.6f32, Color32::GRAY),
                                StrokeKind::Inside,
                            );
                            if *on {
                                cp.line_segment(
                                    [square.min + vec2(1., 3.), square.min + vec2(3., 5.)],
                                    Stroke::new(1f32, Color32::BLACK),
                                );
                                cp.line_segment(
                                    [square.min + vec2(3., 5.), square.min + vec2(6., 1.)],
                                    Stroke::new(1f32, Color32::BLACK),
                                );
                            }
                            if hit.clicked()
                                && hit.interact_pointer_pos().is_some_and(|p| r.contains(p))
                            {
                                events.push(Event::Toggle(*row, column));
                            }
                        }
                        Cell::Check(None) => {}
                    }
                    p.line_segment(
                        [r.right_top(), r.right_bottom()],
                        Stroke::new(0.5f32, Color32::from_gray(190)),
                    );
                }
                p.line_segment(
                    [rr.left_bottom(), rr.right_bottom()],
                    Stroke::new(0.5f32, Color32::from_gray(190)),
                );
            }
            let mut x = rect.left();
            for (i, w) in widths.iter_mut().enumerate() {
                x += *w;
                let r = Rect::from_min_size(pos2(x - 3., rect.top()), vec2(6., 18.));
                let response = ui
                    .interact(r, id.with(("column", i)), Sense::drag())
                    .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
                if response.dragged() {
                    *w = (*w + response.drag_delta().x).max(25.);
                }
            }
        });
    ui.ctx().data_mut(|d| d.insert_temp(id, widths));
    events
}
